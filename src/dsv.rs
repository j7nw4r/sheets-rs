use anyhow::Result;

use crate::app::App;

/// Detect delimiter from file extension.
pub fn detect_delimiter(path: &str) -> u8 {
    if path.ends_with(".tsv") || path.ends_with(".tab") {
        b'\t'
    } else {
        b','
    }
}

/// Load a CSV/TSV file into the app.
pub fn load_file(app: &mut App, path: &str) -> Result<()> {
    let contents = std::fs::read_to_string(path)?;
    let delimiter = detect_delimiter(path);
    app.delimiter = delimiter;
    app.file_path = Some(path.to_string());
    load_with_delimiter(app, contents.as_bytes(), delimiter)
}

/// Load CSV data from a string (stdin).
pub fn load_string(app: &mut App, input: &str) -> Result<()> {
    load_with_delimiter(app, input.as_bytes(), b',')
}

/// Load data with a specific delimiter.
fn load_with_delimiter(app: &mut App, data: &[u8], delimiter: u8) -> Result<()> {
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .flexible(true)
        .from_reader(data);

    let mut max_col: usize = 0;
    let mut row: usize = 0;

    for result in reader.records() {
        let record = result?;
        for (col, field) in record.iter().enumerate() {
            if !field.is_empty() {
                app.set_cell_value(row, col, field.to_string());
            }
            if col >= max_col {
                max_col = col + 1;
            }
        }
        row += 1;
    }

    if row > app.row_count {
        app.row_count = row;
    }
    if max_col > app.col_count {
        app.col_count = max_col;
    }

    Ok(())
}

/// Save the spreadsheet to a CSV/TSV file.
pub fn save_file(app: &App, path: &str) -> Result<()> {
    let delimiter = detect_delimiter(path);
    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_path(path)?;

    // Find the actual data bounds
    let (max_row, max_col) = data_bounds(app);

    for row in 0..=max_row {
        let mut record = Vec::new();
        for col in 0..=max_col {
            record.push(app.cell_value(row, col).to_string());
        }
        writer.write_record(&record)?;
    }

    writer.flush()?;
    Ok(())
}

/// Find the maximum row and column that contain data.
fn data_bounds(app: &App) -> (usize, usize) {
    let mut max_row: usize = 0;
    let mut max_col: usize = 0;
    for key in app.cells.keys() {
        if key.row > max_row {
            max_row = key.row;
        }
        if key.col > max_col {
            max_col = key.col;
        }
    }
    (max_row, max_col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_delimiter() {
        assert_eq!(detect_delimiter("test.csv"), b',');
        assert_eq!(detect_delimiter("test.tsv"), b'\t');
        assert_eq!(detect_delimiter("test.tab"), b'\t');
        assert_eq!(detect_delimiter("test.txt"), b',');
    }

    #[test]
    fn test_load_csv_string() {
        let mut app = App::new();
        load_string(&mut app, "a,b,c\n1,2,3\n").unwrap();
        assert_eq!(app.cell_value(0, 0), "a");
        assert_eq!(app.cell_value(0, 2), "c");
        assert_eq!(app.cell_value(1, 1), "2");
    }

    #[test]
    fn test_roundtrip() {
        let mut app = App::new();
        app.set_cell_value(0, 0, "hello".into());
        app.set_cell_value(0, 1, "world".into());
        app.set_cell_value(1, 0, "foo".into());

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.csv");
        let path_str = path.to_str().unwrap();

        save_file(&app, path_str).unwrap();

        let mut app2 = App::new();
        load_file(&mut app2, path_str).unwrap();

        assert_eq!(app2.cell_value(0, 0), "hello");
        assert_eq!(app2.cell_value(0, 1), "world");
        assert_eq!(app2.cell_value(1, 0), "foo");
    }
}
