use std::fmt;

/// A cell coordinate in the spreadsheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellKey {
    pub row: usize,
    pub col: usize,
}

impl CellKey {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

/// A rectangular range of cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellRange {
    pub start: CellKey,
    pub end: CellKey,
}

impl CellRange {
    pub fn new(start: CellKey, end: CellKey) -> Self {
        let min_row = start.row.min(end.row);
        let max_row = start.row.max(end.row);
        let min_col = start.col.min(end.col);
        let max_col = start.col.max(end.col);
        Self {
            start: CellKey::new(min_row, min_col),
            end: CellKey::new(max_row, max_col),
        }
    }

    pub fn contains(&self, key: CellKey) -> bool {
        key.row >= self.start.row
            && key.row <= self.end.row
            && key.col >= self.start.col
            && key.col <= self.end.col
    }

    /// Iterate over all cells in this range, row by row.
    pub fn iter(&self) -> impl Iterator<Item = CellKey> {
        let start = self.start;
        let end = self.end;
        (start.row..=end.row)
            .flat_map(move |row| (start.col..=end.col).map(move |col| CellKey::new(row, col)))
    }
}

/// Convert a 0-based column index to a column label (A, B, ..., Z, AA, AB, ...).
pub fn column_label(col: usize) -> String {
    let mut label = String::new();
    let mut n = col;
    loop {
        label.insert(0, (b'A' + (n % 26) as u8) as char);
        if n < 26 {
            break;
        }
        n = n / 26 - 1;
    }
    label
}

/// Convert a column label (e.g. "A", "AA") to a 0-based column index.
/// Returns None if the label is empty or contains non-alpha characters.
pub fn parse_column_label(label: &str) -> Option<usize> {
    if label.is_empty() {
        return None;
    }
    let mut col: usize = 0;
    for ch in label.chars() {
        if !ch.is_ascii_alphabetic() {
            return None;
        }
        col = col * 26 + (ch.to_ascii_uppercase() as usize - 'A' as usize) + 1;
    }
    Some(col - 1)
}

/// Format a cell reference like "A1" (1-based row display).
pub fn cell_ref(key: CellKey) -> String {
    format!("{}{}", column_label(key.col), key.row + 1)
}

impl fmt::Display for CellKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", cell_ref(*self))
    }
}

/// Parse a cell reference like "A1" into a CellKey.
/// Returns None if the string is not a valid cell reference.
pub fn parse_cell_ref(s: &str) -> Option<CellKey> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Split into alpha prefix and numeric suffix
    let alpha_end = s
        .find(|c: char| !c.is_ascii_alphabetic())
        .unwrap_or(s.len());
    let col_str = &s[..alpha_end];
    let row_str = &s[alpha_end..];

    if col_str.is_empty() || row_str.is_empty() {
        return None;
    }

    let col = parse_column_label(col_str)?;
    let row: usize = row_str.parse().ok()?;
    if row == 0 {
        return None; // 1-based display
    }

    Some(CellKey::new(row - 1, col))
}

/// Check if a cell's raw value is a formula (starts with '=').
pub fn is_formula(value: &str) -> bool {
    value.starts_with('=')
}

/// Rewrite cell references in a formula by applying a row/col offset.
/// Used when pasting formulas at a different location.
pub fn rewrite_formula_refs(formula: &str, row_offset: isize, col_offset: isize) -> String {
    if !is_formula(formula) {
        return formula.to_string();
    }

    let mut result = String::with_capacity(formula.len());
    let chars: Vec<char> = formula.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Try to parse a cell reference at current position
        if chars[i].is_ascii_alphabetic() {
            let start = i;
            // Consume alpha chars
            while i < chars.len() && chars[i].is_ascii_alphabetic() {
                i += 1;
            }
            // Consume digit chars
            let digit_start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }

            let token: String = chars[start..i].iter().collect();

            // Check if this is a valid cell ref (has both alpha and digit parts)
            if digit_start > start && i > digit_start {
                if let Some(key) = parse_cell_ref(&token) {
                    let new_row = (key.row as isize + row_offset).max(0) as usize;
                    let new_col = (key.col as isize + col_offset).max(0) as usize;
                    result.push_str(&cell_ref(CellKey::new(new_row, new_col)));
                    continue;
                }
            }
            // Not a cell ref, output as-is
            result.push_str(&token);
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_label() {
        assert_eq!(column_label(0), "A");
        assert_eq!(column_label(1), "B");
        assert_eq!(column_label(25), "Z");
        assert_eq!(column_label(26), "AA");
        assert_eq!(column_label(27), "AB");
        assert_eq!(column_label(51), "AZ");
        assert_eq!(column_label(52), "BA");
        assert_eq!(column_label(701), "ZZ");
        assert_eq!(column_label(702), "AAA");
    }

    #[test]
    fn test_parse_column_label() {
        assert_eq!(parse_column_label("A"), Some(0));
        assert_eq!(parse_column_label("B"), Some(1));
        assert_eq!(parse_column_label("Z"), Some(25));
        assert_eq!(parse_column_label("AA"), Some(26));
        assert_eq!(parse_column_label("AZ"), Some(51));
        assert_eq!(parse_column_label("BA"), Some(52));
        assert_eq!(parse_column_label(""), None);
        assert_eq!(parse_column_label("1"), None);
    }

    #[test]
    fn test_column_label_roundtrip() {
        for i in 0..1000 {
            let label = column_label(i);
            assert_eq!(parse_column_label(&label), Some(i), "Failed for column {i}");
        }
    }

    #[test]
    fn test_parse_cell_ref() {
        assert_eq!(parse_cell_ref("A1"), Some(CellKey::new(0, 0)));
        assert_eq!(parse_cell_ref("B2"), Some(CellKey::new(1, 1)));
        assert_eq!(parse_cell_ref("Z10"), Some(CellKey::new(9, 25)));
        assert_eq!(parse_cell_ref("AA1"), Some(CellKey::new(0, 26)));
        assert_eq!(parse_cell_ref("A0"), None);
        assert_eq!(parse_cell_ref(""), None);
        assert_eq!(parse_cell_ref("A"), None);
        assert_eq!(parse_cell_ref("1"), None);
    }

    #[test]
    fn test_cell_ref_roundtrip() {
        for row in 0..50 {
            for col in 0..50 {
                let key = CellKey::new(row, col);
                let ref_str = cell_ref(key);
                assert_eq!(
                    parse_cell_ref(&ref_str),
                    Some(key),
                    "Failed for ({row}, {col})"
                );
            }
        }
    }

    #[test]
    fn test_rewrite_formula_refs() {
        assert_eq!(rewrite_formula_refs("=A1+B2", 1, 1), "=B2+C3");
        assert_eq!(rewrite_formula_refs("=SUM(A1:A5)", 2, 0), "=SUM(A3:A7)");
        assert_eq!(rewrite_formula_refs("hello", 1, 1), "hello");
        assert_eq!(rewrite_formula_refs("=A1+10", 0, 0), "=A1+10");
    }
}
