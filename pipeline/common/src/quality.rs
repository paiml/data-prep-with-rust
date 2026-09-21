/// A before/after data-quality report, produced by every layer of the
/// medallion pipeline (bronze, silver, gold) so a run can be audited without
/// re-reading the data.
#[derive(Debug, Clone, Default)]
pub struct DataQualityReport {
    pub stage: String,
    pub input: String,
    pub output: String,
    pub rows_in: usize,
    pub rows_out: usize,
    pub duplicates_dropped: usize,
    pub columns_dropped: Vec<String>,
    pub nulls_remaining: usize,
}

impl DataQualityReport {
    pub fn rows_dropped(&self) -> usize {
        self.rows_in.saturating_sub(self.rows_out)
    }

    pub fn print(&self) {
        println!("{} report for {}", self.stage, self.input);
        println!("  output file:        {}", self.output);
        println!(
            "  rows in / out:       {} / {} ({} dropped)",
            self.rows_in,
            self.rows_out,
            self.rows_dropped()
        );
        if self.duplicates_dropped > 0 {
            println!("  duplicates dropped:  {}", self.duplicates_dropped);
        }
        if !self.columns_dropped.is_empty() {
            println!(
                "  columns dropped:     {} ({})",
                self.columns_dropped.len(),
                self.columns_dropped.join(", ")
            );
        }
        println!("  nulls remaining:     {}", self.nulls_remaining);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_dropped_is_the_difference() {
        let report = DataQualityReport {
            rows_in: 100,
            rows_out: 92,
            ..Default::default()
        };
        assert_eq!(report.rows_dropped(), 8);
    }

    #[test]
    fn rows_dropped_never_underflows() {
        let report = DataQualityReport {
            rows_in: 0,
            rows_out: 5,
            ..Default::default()
        };
        assert_eq!(report.rows_dropped(), 0);
    }
}
