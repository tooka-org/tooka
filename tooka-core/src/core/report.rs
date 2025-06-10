//! Report generation module for Tooka.
//!
//! Supports creating reports in JSON, CSV, and PDF formats from sorting results.

use crate::{core::error::TookaError, core::sorter::MatchResult, utils::gen_pdf::generate_pdf};
use anyhow::Result;
use std::{
    fs::{File, create_dir_all},
    path::Path,
};

/// Generates a report from sorting results in the specified format.
///
/// Supported formats are `"json"`, `"csv"`, and `"pdf"`. The generated report
/// is saved in the provided output directory.
///
/// # Arguments
/// * `report_type` - A string slice indicating the desired report format.
/// * `output_dir` - Path to the directory where the report will be saved.
/// * `results` - Slice of [`MatchResult`] structs containing sorting results.
///
/// # Errors
/// Returns a [`TookaError`] if directory creation, file writing, or PDF generation fails,
/// or if an unsupported report format is requested.
pub fn generate_report(
    report_type: &str,
    output_dir: &Path,
    results: &[MatchResult],
) -> Result<(), TookaError> {
    create_dir_all(output_dir)?;

    match report_type.to_lowercase().as_str() {
        "json" => {
            let path = output_dir.join("tooka_report.json");
            let file = File::create(&path)?;
            serde_json::to_writer_pretty(file, results)?
        }
        "csv" => {
            let path = output_dir.join("tooka_report.csv");
            let mut wtr = csv::Writer::from_path(&path)?;
            // Write header
            wtr.write_record([
                "file_name",
                "action",
                "matched_rule_id",
                "current_path",
                "new_path",
            ])?;
            for r in results {
                wtr.serialize((
                    &r.file_name,
                    &r.action,
                    &r.matched_rule_id,
                    r.current_path.display().to_string(),
                    r.new_path.display().to_string(),
                ))?;
            }
            wtr.flush()?
        }
        "pdf" => {
            let path = output_dir.join("tooka_report.pdf");
            generate_pdf(&path, results)
                .map_err(|e| TookaError::PdfGenerationError(e.to_string()))?;
        }
        other => {
            return Err(TookaError::Other(format!(
                "Unsupported report format: {}",
                other
            )));
        }
    }

    Ok(())
}
