use std::collections::HashMap;

/// Per-column running statistics collected in a single pass over a CSV file.
pub struct ColumnStats {
    pub name: String,
    pub count: usize,
    pub missing: usize,
    pub numeric_count: usize,
    pub integer_count: usize,
    pub sum: f64,
    pub sum_sq: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub value_counts: HashMap<String, usize>,
    distinct_cap: usize,
    pub distinct_overflow: bool,
}

impl ColumnStats {
    pub fn new(name: String) -> Self {
        ColumnStats {
            name,
            count: 0,
            missing: 0,
            numeric_count: 0,
            integer_count: 0,
            sum: 0.0,
            sum_sq: 0.0,
            min: None,
            max: None,
            value_counts: HashMap::new(),
            distinct_cap: 10_000,
            distinct_overflow: false,
        }
    }

    pub fn observe(&mut self, raw: &str) {
        self.count += 1;
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            self.missing += 1;
            return;
        }

        let normalized = trimmed.replace('_', "");
        if let Ok(n) = normalized.parse::<f64>() {
            if n.is_finite() {
                self.numeric_count += 1;
                self.sum += n;
                self.sum_sq += n * n;
                self.min = Some(self.min.map_or(n, |m| m.min(n)));
                self.max = Some(self.max.map_or(n, |m| m.max(n)));

                let looks_integer = !normalized.contains('.') && !normalized.contains(['e', 'E']);
                if looks_integer && normalized.parse::<i64>().is_ok() {
                    self.integer_count += 1;
                }
            }
        }

        if self.value_counts.len() < self.distinct_cap || self.value_counts.contains_key(trimmed) {
            *self.value_counts.entry(trimmed.to_string()).or_insert(0) += 1;
        } else {
            self.distinct_overflow = true;
        }
    }

    pub fn present(&self) -> usize {
        self.count - self.missing
    }

    /// A column is treated as numeric when every non-missing value parsed as a number.
    pub fn is_numeric(&self) -> bool {
        self.present() > 0 && self.numeric_count == self.present()
    }

    /// A numeric column is treated as an integer column when every numeric value
    /// parsed as a whole number (no decimal point or exponent).
    pub fn is_integer(&self) -> bool {
        self.numeric_count > 0 && self.integer_count == self.numeric_count
    }

    pub fn mean(&self) -> Option<f64> {
        if self.numeric_count == 0 {
            None
        } else {
            Some(self.sum / self.numeric_count as f64)
        }
    }

    pub fn distinct_count(&self) -> usize {
        self.value_counts.len()
    }

    /// True when every value observed in this column is identical (and at
    /// least one value was observed) — the column carries no information.
    pub fn is_constant(&self) -> bool {
        self.present() > 0 && self.distinct_count() == 1 && !self.distinct_overflow
    }
}

pub struct TableSummary {
    pub file: String,
    pub rows: usize,
    pub columns: Vec<ColumnStats>,
}

/// Read a CSV file and collect per-column statistics in a single pass.
pub fn summarize_csv(
    path: &std::path::Path,
    delim_byte: u8,
    has_header: bool,
) -> anyhow::Result<TableSummary> {
    use anyhow::Context;

    let file =
        std::fs::File::open(path).with_context(|| format!("failed to open {}", path.display()))?;

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delim_byte)
        .has_headers(has_header)
        .flexible(true)
        .from_reader(file);

    let headers: Vec<String> = if has_header {
        reader
            .headers()
            .context("failed to read CSV header row")?
            .iter()
            .map(|h| h.to_string())
            .collect()
    } else {
        Vec::new()
    };

    let mut columns: Vec<ColumnStats> = headers
        .iter()
        .map(|h| ColumnStats::new(h.clone()))
        .collect();

    let mut rows = 0usize;
    for result in reader.records() {
        let record =
            result.with_context(|| format!("failed to read a row in {}", path.display()))?;

        while columns.len() < record.len() {
            let idx = columns.len() + 1;
            columns.push(ColumnStats::new(format!("column_{idx}")));
        }

        for (i, col) in columns.iter_mut().enumerate() {
            let raw = record.get(i).unwrap_or("");
            col.observe(raw);
        }
        rows += 1;
    }

    Ok(TableSummary {
        file: path.display().to_string(),
        rows,
        columns,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observe_tracks_missing_and_numeric() {
        let mut col = ColumnStats::new("age".to_string());
        col.observe("10");
        col.observe("");
        col.observe("20");

        assert_eq!(col.count, 3);
        assert_eq!(col.missing, 1);
        assert!(col.is_numeric());
        assert!(col.is_integer());
        assert_eq!(col.mean(), Some(15.0));
    }

    #[test]
    fn is_constant_true_only_for_a_single_distinct_value() {
        let mut col = ColumnStats::new("flag".to_string());
        col.observe("yes");
        col.observe("yes");
        assert!(col.is_constant());

        col.observe("no");
        assert!(!col.is_constant());
    }

    #[test]
    fn mixed_type_column_is_not_numeric() {
        let mut col = ColumnStats::new("mixed".to_string());
        col.observe("10");
        col.observe("abc");
        assert!(!col.is_numeric());
    }
}
