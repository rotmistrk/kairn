//! CSV parsing — delimiter detection, quote-aware splitting, type detection, serialization.

/// Column type classification.
#[derive(Debug, Clone)]
pub enum ColType {
    Text,
    Numeric {
        max_before_dot: u16,
        max_after_dot: u16,
        /// Maximum width of the exponent part (e.g. "e+07" = 4). 0 if no scientific values.
        max_exp_width: u16,
    },
}

/// Parsed CSV data.
pub struct CsvData {
    pub(crate) delimiter: char,
    pub(crate) headers: Option<Vec<String>>,
    pub(crate) rows: Vec<Vec<String>>,
    pub(crate) col_types: Vec<ColType>,
}

/// Detect delimiter from first few lines.
pub fn detect_delimiter(text: &str) -> char {
    let candidates = [',', '\t', '|', ';'];
    let lines: Vec<&str> = text.lines().take(5).collect();
    if lines.is_empty() {
        return ',';
    }
    let mut best = ',';
    let mut best_score = 0usize;
    for &delim in &candidates {
        let counts: Vec<usize> = lines.iter().map(|l| count_unquoted(l, delim)).collect();
        if counts.is_empty() || counts[0] == 0 {
            continue;
        }
        // Consistent count across lines = good delimiter
        let consistent = counts.iter().all(|&c| c == counts[0]);
        let score = if consistent {
            counts[0] * 10
        } else {
            counts[0]
        };
        if score > best_score {
            best_score = score;
            best = delim;
        }
    }
    best
}

/// Parse CSV text into rows of fields.
pub fn parse(text: &str, delimiter: char) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut chars = text.chars().peekable();
    loop {
        if chars.peek().is_none() {
            break;
        }
        let row = parse_row(&mut chars, delimiter);
        rows.push(row);
    }
    rows
}

/// Detect if first row is a header.
pub fn detect_header(rows: &[Vec<String>]) -> bool {
    if rows.len() < 2 {
        return false;
    }
    let first = &rows[0];
    // Header if first row has no purely numeric fields
    !first.iter().any(|f| f.parse::<f64>().is_ok() && !f.is_empty())
}

/// Classify column types from data rows.
pub fn detect_col_types(rows: &[Vec<String>], has_header: bool) -> Vec<ColType> {
    let data_start = if has_header {
        1
    } else {
        0
    };
    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    (0..ncols).map(|col| classify_column(rows, col, data_start)).collect()
}

/// Full parse: detect delimiter, parse, detect header and types.
pub fn parse_csv(text: &str) -> CsvData {
    let delimiter = detect_delimiter(text);
    let mut rows = parse(text, delimiter);
    let has_header = detect_header(&rows);
    let col_types = detect_col_types(&rows, has_header);
    let headers = if has_header && !rows.is_empty() {
        Some(rows.remove(0))
    } else {
        None
    };
    CsvData {
        delimiter,
        headers,
        rows,
        col_types,
    }
}

/// Serialize rows back to CSV text.
pub fn serialize(headers: Option<&[String]>, rows: &[Vec<String>], delimiter: char) -> String {
    let mut out = String::new();
    if let Some(hdrs) = headers {
        serialize_row(&mut out, hdrs, delimiter);
    }
    for row in rows {
        serialize_row(&mut out, row, delimiter);
    }
    out
}

fn serialize_row(out: &mut String, fields: &[String], delimiter: char) {
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            out.push(delimiter);
        }
        if field.contains(delimiter) || field.contains('"') || field.contains('\n') {
            out.push('"');
            out.push_str(&field.replace('"', "\"\""));
            out.push('"');
        } else {
            out.push_str(field);
        }
    }
    out.push('\n');
}

fn count_unquoted(line: &str, delim: char) -> usize {
    let mut count = 0;
    let mut in_quote = false;
    for ch in line.chars() {
        match ch {
            '"' => in_quote = !in_quote,
            c if c == delim && !in_quote => count += 1,
            _ => {}
        }
    }
    count
}

fn parse_row(chars: &mut std::iter::Peekable<std::str::Chars>, delimiter: char) -> Vec<String> {
    let mut fields = Vec::new();
    loop {
        let field = parse_field(chars, delimiter);
        fields.push(field);
        match chars.peek() {
            Some(&c) if c == delimiter => {
                chars.next();
            }
            Some(&'\n') => {
                chars.next();
                break;
            }
            Some(&'\r') => {
                chars.next();
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                break;
            }
            None => break,
            _ => break,
        }
    }
    fields
}

fn parse_field(chars: &mut std::iter::Peekable<std::str::Chars>, delimiter: char) -> String {
    if chars.peek() == Some(&'"') {
        chars.next(); // consume opening quote
        parse_quoted_field(chars)
    } else {
        parse_unquoted_field(chars, delimiter)
    }
}

fn parse_quoted_field(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut field = String::new();
    loop {
        match chars.next() {
            Some('"') if chars.peek() == Some(&'"') => {
                chars.next();
                field.push('"');
            }
            Some('"') => break,
            Some(c) => field.push(c),
            None => break,
        }
    }
    field
}

fn parse_unquoted_field(chars: &mut std::iter::Peekable<std::str::Chars>, delimiter: char) -> String {
    let mut field = String::new();
    loop {
        match chars.peek() {
            Some(&c) if c == delimiter || c == '\n' || c == '\r' => break,
            Some(_) => field.push(chars.next().unwrap_or(' ')),
            None => break,
        }
    }
    field
}

fn is_special_value(s: &str) -> bool {
    matches!(s.trim(), "" | "-" | "*" | "n/a" | "N/A" | "NA" | "na" | "null" | "NULL")
}

fn classify_column(rows: &[Vec<String>], col: usize, data_start: usize) -> ColType {
    let mut numeric_count = 0usize;
    let mut total_count = 0usize;
    let mut max_before = 0u16;
    let mut max_after = 0u16;
    let mut max_exp = 0u16;

    for row in rows.iter().skip(data_start) {
        let val = row.get(col).map(|s| s.as_str()).unwrap_or("");
        if is_special_value(val) {
            continue;
        }
        total_count += 1;
        let trimmed = val.trim();
        if trimmed.parse::<f64>().is_ok() {
            numeric_count += 1;
            // Split off scientific exponent part (e/E)
            let (mantissa, exp_part) = split_scientific(trimmed);
            max_exp = max_exp.max(exp_part.len() as u16);
            if let Some(dot) = mantissa.find('.') {
                let before = dot as u16;
                let after = (mantissa.len() - dot - 1) as u16;
                max_before = max_before.max(before);
                max_after = max_after.max(after);
            } else {
                max_before = max_before.max(mantissa.len() as u16);
            }
        }
    }

    if total_count > 0 && numeric_count * 5 >= total_count * 4 {
        ColType::Numeric {
            max_before_dot: max_before,
            max_after_dot: max_after,
            max_exp_width: max_exp,
        }
    } else {
        ColType::Text
    }
}

/// Split a numeric string into (mantissa, exponent_suffix).
/// E.g. "1.23e+07" → ("1.23", "e+07"), "42" → ("42", ""), "-3.14" → ("-3.14", "")
fn split_scientific(s: &str) -> (&str, &str) {
    // Find 'e' or 'E' that's part of scientific notation (not at start)
    for (i, ch) in s.char_indices() {
        if i > 0 && (ch == 'e' || ch == 'E') {
            return (&s[..i], &s[i..]);
        }
    }
    (s, "")
}
