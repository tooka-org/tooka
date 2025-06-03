use crate::core::{common::gen_pdf::generate_pdf, sorter::MatchResult};
use anyhow::{Result, bail};
use std::{
    fs::{File, create_dir_all},
    path::Path,
};

pub fn generate_report(
    report_type: &str,
    output_dir: &Path,
    results: &[MatchResult],
) -> Result<()> {
    create_dir_all(output_dir)?;

    match report_type.to_lowercase().as_str() {
        "json" => {
            let path = output_dir.join("tooka_report.json");
            let file = File::create(&path)?;
            serde_json::to_writer_pretty(file, results)?;
            println!("✅ JSON report written to {}", path.display());
        }
        "csv" => {
            let path = output_dir.join("tooka_report.csv");
            let mut wtr = csv::Writer::from_path(&path)?;
            for r in results {
                wtr.serialize((
                    &r.file_name,
                    &r.matched_rule_id,
                    r.current_path.display().to_string(),
                    r.new_path.display().to_string(),
                ))?;
            }
            wtr.flush()?;
            println!("✅ CSV report written to {}", path.display());
        }
        "pdf" => {
            let path = output_dir.join("tooka_report.pdf");
            generate_pdf(&path, results)?;
            println!("✅ PDF report written to {}", path.display());
        }
        other => bail!("Unsupported report format: {}", other),
    }

    Ok(())
}
