use crate::config::FilterConfig;
use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashSet;

pub fn apply_filters(log: &str, filters: &[FilterConfig]) -> Result<String> {
    let mut lines: Vec<&str> = log.lines().collect();

    for filter in filters {
        match filter {
            FilterConfig::Head { n } => {
                if *n < lines.len() {
                    lines.truncate(*n);
                }
            }
            FilterConfig::Tail { n } => {
                if *n < lines.len() {
                    let start = lines.len() - n;
                    lines = lines[start..].to_vec();
                }
            }
            FilterConfig::Grep {
                pattern,
                before,
                after,
            } => {
                let re = Regex::new(pattern).context("Invalid regex pattern")?;
                let mut keep_indices = HashSet::new();

                for (i, line) in lines.iter().enumerate() {
                    if re.is_match(line) {
                        let start = i.saturating_sub(*before);
                        let end = (i + after + 1).min(lines.len());
                        for k in start..end {
                            keep_indices.insert(k);
                        }
                    }
                }

                let mut sorted_indices: Vec<usize> = keep_indices.into_iter().collect();
                sorted_indices.sort();

                let mut new_lines = Vec::new();
                for idx in sorted_indices {
                    new_lines.push(lines[idx]);
                }
                lines = new_lines;
            }
        }
    }

    Ok(lines.join("\n"))
}
