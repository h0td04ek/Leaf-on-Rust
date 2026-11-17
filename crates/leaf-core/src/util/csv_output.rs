//! Перенос `net.minecraft.util.CsvOutput`.

use std::io::Write;

const LINE_SEPARATOR: &str = "\r\n";
const FIELD_SEPARATOR: &str = ",";

/// `CsvOutput` — CSV-писатель с фиксированным числом колонок.
pub struct CsvOutput<W: Write> {
    output: W,
    column_count: usize,
}

impl<W: Write> CsvOutput<W> {
    fn new(mut output: W, first_row: &[String]) -> Result<Self, std::io::Error> {
        let column_count = first_row.len();
        Self::write_line_raw(&mut output, first_row.iter().map(|s| s.as_str()))?;
        Ok(Self { output, column_count })
    }

    /// `builder()`.
    pub fn builder() -> CsvOutputBuilder {
        CsvOutputBuilder::new()
    }

    /// `writeRow(data...)` — число колонок должно совпадать.
    pub fn write_row(&mut self, data: &[String]) -> Result<(), CsvOutputError> {
        if data.len() != self.column_count {
            return Err(CsvOutputError::ColumnCount {
                expected: self.column_count,
                got: data.len(),
            });
        }
        self.write_line(data.iter().map(|s| escape_csv(s)))
            .map_err(CsvOutputError::Io)
    }

    fn write_line<'a>(&mut self, data: impl Iterator<Item = String> + 'a) -> std::io::Result<()> {
        let line = data.collect::<Vec<_>>().join(FIELD_SEPARATOR);
        write!(self.output, "{}{}", line, LINE_SEPARATOR)
    }

    fn write_line_raw<'a>(output: &mut W, data: impl Iterator<Item = &'a str>) -> std::io::Result<()> {
        let line = data.collect::<Vec<_>>().join(FIELD_SEPARATOR);
        write!(output, "{}{}", line, LINE_SEPARATOR)
    }
}

/// `CsvOutput.Builder`.
pub struct CsvOutputBuilder {
    headers: Vec<String>,
}

impl CsvOutputBuilder {
    pub fn new() -> Self {
        Self { headers: Vec::new() }
    }

    /// `addColumn(header)`.
    pub fn add_column(&mut self, header: &str) -> &mut Self {
        self.headers.push(header.to_string());
        self
    }

    /// `build(output)`.
    pub fn build<W: Write>(self, output: W) -> Result<CsvOutput<W>, std::io::Error> {
        CsvOutput::new(output, &self.headers)
    }
}

impl Default for CsvOutputBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum CsvOutputError {
    ColumnCount { expected: usize, got: usize },
    Io(std::io::Error),
}

impl std::fmt::Display for CsvOutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ColumnCount { expected, got } => {
                write!(f, "Invalid number of columns, expected {}, but got {}", expected, got)
            }
            Self::Io(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for CsvOutputError {}

/// `StringEscapeUtils.escapeCsv` — Apache Commons Lang.
fn escape_csv(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}
