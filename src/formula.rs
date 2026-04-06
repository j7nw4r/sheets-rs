//! Recursive descent formula parser and evaluator.
//!
//! Formulas start with '=' and support:
//! - Arithmetic: +, -, *, /
//! - Cell references: A1, B2
//! - Ranges: A1:B5
//! - Functions: SUM, AVG, MIN, MAX, COUNT
//! - Parentheses for grouping
//! - Circular dependency detection

use std::collections::HashSet;

use crate::app::App;
use crate::cell::{CellKey, parse_cell_ref, parse_column_label, is_formula};

/// The result of evaluating a formula.
#[derive(Debug, Clone)]
enum FormulaValue {
    Number(f64),
    Text(String),
    Blank,
    Error(String),
    /// Multiple values from a range expansion (used inside aggregate functions).
    Range(Vec<f64>),
}

impl FormulaValue {
    fn as_number(&self) -> Option<f64> {
        match self {
            FormulaValue::Number(n) => Some(*n),
            FormulaValue::Text(s) => s.parse::<f64>().ok(),
            FormulaValue::Blank => Some(0.0),
            FormulaValue::Range(vals) => Some(vals.iter().sum()),
            FormulaValue::Error(_) => None,
        }
    }

    /// Flatten into individual numeric values (for aggregate functions).
    fn into_numbers(self) -> Vec<f64> {
        match self {
            FormulaValue::Number(n) => vec![n],
            FormulaValue::Text(s) => s.parse::<f64>().ok().into_iter().collect(),
            FormulaValue::Blank => vec![],
            FormulaValue::Range(vals) => vals,
            FormulaValue::Error(_) => vec![],
        }
    }

    fn to_display(&self) -> String {
        match self {
            FormulaValue::Number(n) => {
                if *n == n.floor() && n.abs() < 1e15 {
                    format!("{}", *n as i64)
                } else {
                    format!("{n}")
                }
            }
            FormulaValue::Text(s) => s.clone(),
            FormulaValue::Blank => String::new(),
            FormulaValue::Range(vals) => format!("{}", vals.iter().sum::<f64>()),
            FormulaValue::Error(e) => format!("#{e}"),
        }
    }
}

/// Context for formula evaluation, tracking visited cells for cycle detection.
struct EvalContext {
    visiting: HashSet<CellKey>,
}

impl EvalContext {
    fn new() -> Self {
        Self {
            visiting: HashSet::new(),
        }
    }
}

/// Parser for formula expressions.
struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
    app: &'a App,
    ctx: &'a mut EvalContext,
    current: CellKey,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str, app: &'a App, ctx: &'a mut EvalContext, current: CellKey) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
            app,
            ctx,
            current,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let ch = self.peek()?;
        self.pos += 1;
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == b' ' || ch == b'\t' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Parse a full expression (lowest precedence: addition, subtraction).
    fn parse_expression(&mut self) -> FormulaValue {
        let mut left = self.parse_term();

        loop {
            self.skip_whitespace();
            match self.peek() {
                Some(b'+') => {
                    self.advance();
                    let right = self.parse_term();
                    left = self.binary_op(left, right, |a, b| a + b);
                }
                Some(b'-') => {
                    self.advance();
                    let right = self.parse_term();
                    left = self.binary_op(left, right, |a, b| a - b);
                }
                _ => break,
            }
        }

        left
    }

    /// Parse a term (multiplication, division).
    fn parse_term(&mut self) -> FormulaValue {
        let mut left = self.parse_unary();

        loop {
            self.skip_whitespace();
            match self.peek() {
                Some(b'*') => {
                    self.advance();
                    let right = self.parse_unary();
                    left = self.binary_op(left, right, |a, b| a * b);
                }
                Some(b'/') => {
                    self.advance();
                    let right = self.parse_unary();
                    match right.as_number() {
                        Some(0.0) => return FormulaValue::Error("DIV/0".into()),
                        _ => left = self.binary_op(left, right, |a, b| a / b),
                    }
                }
                _ => break,
            }
        }

        left
    }

    /// Parse unary +/-.
    fn parse_unary(&mut self) -> FormulaValue {
        self.skip_whitespace();
        match self.peek() {
            Some(b'+') => {
                self.advance();
                self.parse_unary()
            }
            Some(b'-') => {
                self.advance();
                let val = self.parse_unary();
                match val.as_number() {
                    Some(n) => FormulaValue::Number(-n),
                    None => FormulaValue::Error("VALUE".into()),
                }
            }
            _ => self.parse_primary(),
        }
    }

    /// Parse a primary value: number, cell ref, function call, or parenthesized expression.
    fn parse_primary(&mut self) -> FormulaValue {
        self.skip_whitespace();

        match self.peek() {
            Some(b'(') => {
                self.advance();
                let val = self.parse_expression();
                self.skip_whitespace();
                if self.peek() == Some(b')') {
                    self.advance();
                }
                val
            }
            Some(ch) if ch.is_ascii_digit() || ch == b'.' => self.parse_number(),
            Some(ch) if ch.is_ascii_alphabetic() => self.parse_identifier(),
            Some(b'"') => self.parse_string(),
            _ => FormulaValue::Blank,
        }
    }

    fn parse_number(&mut self) -> FormulaValue {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == b'.' {
                self.advance();
            } else {
                break;
            }
        }
        let s = std::str::from_utf8(&self.input[start..self.pos]).unwrap_or("0");
        match s.parse::<f64>() {
            Ok(n) => FormulaValue::Number(n),
            Err(_) => FormulaValue::Error("VALUE".into()),
        }
    }

    fn parse_string(&mut self) -> FormulaValue {
        self.advance(); // skip opening quote
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch == b'"' {
                let s = std::str::from_utf8(&self.input[start..self.pos])
                    .unwrap_or("")
                    .to_string();
                self.advance();
                return FormulaValue::Text(s);
            }
            self.advance();
        }
        let s = std::str::from_utf8(&self.input[start..self.pos])
            .unwrap_or("")
            .to_string();
        FormulaValue::Text(s)
    }

    fn parse_identifier(&mut self) -> FormulaValue {
        let start = self.pos;

        // Consume alphabetic chars
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphabetic() {
                self.advance();
            } else {
                break;
            }
        }

        let alpha_part = std::str::from_utf8(&self.input[start..self.pos])
            .unwrap_or("")
            .to_uppercase();

        self.skip_whitespace();

        // Check if this is a function call
        if self.peek() == Some(b'(') {
            return self.parse_function_call(&alpha_part);
        }

        // Check if digits follow -> cell reference
        let digit_start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        if self.pos > digit_start {
            // This is a cell reference like A1
            let full_ref = format!(
                "{}{}",
                alpha_part,
                std::str::from_utf8(&self.input[digit_start..self.pos]).unwrap_or("")
            );

            // Check for range operator ':'
            self.skip_whitespace();
            if self.peek() == Some(b':') {
                self.advance();
                return self.parse_range_end(&full_ref);
            }

            // Single cell reference
            if let Some(key) = parse_cell_ref(&full_ref) {
                return self.eval_cell_ref(key);
            }
            return FormulaValue::Error("REF".into());
        }

        // Check for column range like A:A
        self.skip_whitespace();
        if self.peek() == Some(b':') {
            self.advance();
            return self.parse_column_range(&alpha_part);
        }

        // Unknown identifier
        FormulaValue::Error("NAME".into())
    }

    fn parse_function_call(&mut self, name: &str) -> FormulaValue {
        self.advance(); // skip '('

        // Collect argument values for aggregate functions
        let mut values: Vec<f64> = Vec::new();
        self.collect_args(&mut values);

        self.skip_whitespace();
        if self.peek() == Some(b')') {
            self.advance();
        }

        match name {
            "SUM" => FormulaValue::Number(values.iter().sum()),
            "AVG" | "AVERAGE" => {
                if values.is_empty() {
                    FormulaValue::Error("DIV/0".into())
                } else {
                    FormulaValue::Number(values.iter().sum::<f64>() / values.len() as f64)
                }
            }
            "MIN" => values
                .iter()
                .copied()
                .reduce(f64::min)
                .map(FormulaValue::Number)
                .unwrap_or(FormulaValue::Number(0.0)),
            "MAX" => values
                .iter()
                .copied()
                .reduce(f64::max)
                .map(FormulaValue::Number)
                .unwrap_or(FormulaValue::Number(0.0)),
            "COUNT" => FormulaValue::Number(values.len() as f64),
            _ => FormulaValue::Error("NAME".into()),
        }
    }

    fn collect_args(&mut self, values: &mut Vec<f64>) {
        loop {
            self.skip_whitespace();
            if self.peek() == Some(b')') || self.at_end() {
                break;
            }

            let val = self.parse_expression();
            values.extend(val.into_numbers());

            self.skip_whitespace();
            if self.peek() == Some(b',') {
                self.advance();
            }
        }
    }

    fn parse_range_end(&mut self, start_ref: &str) -> FormulaValue {
        self.skip_whitespace();

        // Parse end reference
        let end_start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() {
                self.advance();
            } else {
                break;
            }
        }

        let end_ref = std::str::from_utf8(&self.input[end_start..self.pos])
            .unwrap_or("")
            .to_uppercase();

        let start_key = match parse_cell_ref(start_ref) {
            Some(k) => k,
            None => return FormulaValue::Error("REF".into()),
        };
        let end_key = match parse_cell_ref(&end_ref) {
            Some(k) => k,
            None => return FormulaValue::Error("REF".into()),
        };

        // Evaluate all cells in range and collect numeric values
        let range = crate::cell::CellRange::new(start_key, end_key);
        let mut values = Vec::new();
        for key in range.iter() {
            let val = self.eval_cell_ref(key);
            if let Some(n) = val.as_number() {
                values.push(n);
            }
        }

        FormulaValue::Range(values)
    }

    fn parse_column_range(&mut self, col_label: &str) -> FormulaValue {
        // Column range like A:A — sum the entire column
        self.skip_whitespace();
        let end_start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphabetic() {
                self.advance();
            } else {
                break;
            }
        }
        let end_label = std::str::from_utf8(&self.input[end_start..self.pos])
            .unwrap_or("")
            .to_uppercase();

        let start_col = match parse_column_label(col_label) {
            Some(c) => c,
            None => return FormulaValue::Error("REF".into()),
        };
        let end_col = match parse_column_label(&end_label) {
            Some(c) => c,
            None => return FormulaValue::Error("REF".into()),
        };

        let mut sum = 0.0;
        for row in 0..self.app.row_count {
            for col in start_col..=end_col {
                let val = self.eval_cell_ref(CellKey::new(row, col));
                if let Some(n) = val.as_number() {
                    sum += n;
                }
            }
        }
        FormulaValue::Number(sum)
    }

    fn eval_cell_ref(&mut self, key: CellKey) -> FormulaValue {
        // Circular dependency check
        if self.ctx.visiting.contains(&key) {
            return FormulaValue::Error("CIRC".into());
        }

        let raw = self.app.cell_value(key.row, key.col);
        if raw.is_empty() {
            return FormulaValue::Blank;
        }

        if is_formula(raw) {
            let formula = raw[1..].to_string();
            self.ctx.visiting.insert(key);
            let mut sub_parser = Parser::new(&formula, self.app, self.ctx, key);
            let result = sub_parser.parse_expression();
            self.ctx.visiting.remove(&key);
            result
        } else {
            match raw.parse::<f64>() {
                Ok(n) => FormulaValue::Number(n),
                Err(_) => FormulaValue::Text(raw.to_string()),
            }
        }
    }

    fn binary_op(
        &self,
        left: FormulaValue,
        right: FormulaValue,
        op: impl Fn(f64, f64) -> f64,
    ) -> FormulaValue {
        match (left.as_number(), right.as_number()) {
            (Some(a), Some(b)) => FormulaValue::Number(op(a, b)),
            _ => FormulaValue::Error("VALUE".into()),
        }
    }
}

/// Evaluate a cell's formula and return the display string.
pub fn evaluate_cell(app: &App, row: usize, col: usize) -> String {
    let raw = app.cell_value(row, col);
    if !is_formula(raw) {
        return raw.to_string();
    }

    let formula = &raw[1..];
    let current = CellKey::new(row, col);
    let mut ctx = EvalContext::new();
    ctx.visiting.insert(current);

    let mut parser = Parser::new(formula, app, &mut ctx, current);
    let result = parser.parse_expression();
    result.to_display()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(app: &App, row: usize, col: usize) -> String {
        evaluate_cell(app, row, col)
    }

    #[test]
    fn test_arithmetic() {
        let mut app = App::new();
        app.set_cell_value(0, 0, "=1+2".into());
        assert_eq!(eval(&app, 0, 0), "3");

        app.set_cell_value(0, 1, "=10-3".into());
        assert_eq!(eval(&app, 0, 1), "7");

        app.set_cell_value(0, 2, "=4*5".into());
        assert_eq!(eval(&app, 0, 2), "20");

        app.set_cell_value(0, 3, "=10/4".into());
        assert_eq!(eval(&app, 0, 3), "2.5");
    }

    #[test]
    fn test_precedence() {
        let mut app = App::new();
        app.set_cell_value(0, 0, "=2+3*4".into());
        assert_eq!(eval(&app, 0, 0), "14");

        app.set_cell_value(0, 1, "=(2+3)*4".into());
        assert_eq!(eval(&app, 0, 1), "20");
    }

    #[test]
    fn test_cell_references() {
        let mut app = App::new();
        app.set_cell_value(0, 0, "10".into());
        app.set_cell_value(0, 1, "20".into());
        app.set_cell_value(0, 2, "=A1+B1".into());
        assert_eq!(eval(&app, 0, 2), "30");
    }

    #[test]
    fn test_circular_dependency() {
        let mut app = App::new();
        app.set_cell_value(0, 0, "=B1".into());
        app.set_cell_value(0, 1, "=A1".into());
        assert_eq!(eval(&app, 0, 0), "#CIRC");
    }

    #[test]
    fn test_division_by_zero() {
        let mut app = App::new();
        app.set_cell_value(0, 0, "=1/0".into());
        assert_eq!(eval(&app, 0, 0), "#DIV/0");
    }

    #[test]
    fn test_sum_function() {
        let mut app = App::new();
        app.set_cell_value(0, 0, "1".into());
        app.set_cell_value(1, 0, "2".into());
        app.set_cell_value(2, 0, "3".into());
        app.set_cell_value(3, 0, "=SUM(A1:A3)".into());
        assert_eq!(eval(&app, 3, 0), "6");
    }

    #[test]
    fn test_avg_function() {
        let mut app = App::new();
        app.set_cell_value(0, 0, "10".into());
        app.set_cell_value(1, 0, "20".into());
        app.set_cell_value(2, 0, "30".into());
        app.set_cell_value(3, 0, "=AVG(A1:A3)".into());
        assert_eq!(eval(&app, 3, 0), "20");
    }

    #[test]
    fn test_unary_minus() {
        let mut app = App::new();
        app.set_cell_value(0, 0, "=-5".into());
        assert_eq!(eval(&app, 0, 0), "-5");

        app.set_cell_value(0, 1, "=-5+10".into());
        assert_eq!(eval(&app, 0, 1), "5");
    }

    #[test]
    fn test_blank_cell_as_zero() {
        let mut app = App::new();
        app.set_cell_value(0, 0, "=A2+5".into());
        assert_eq!(eval(&app, 0, 0), "5");
    }
}
