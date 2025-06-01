use crate::core::sorter::MatchResult;
use anyhow::{Result, bail};
use genpdf::{
    Alignment, Document, Element,
    elements::{Break, Image, Paragraph, TableLayout},
    fonts::Builtin,
    style,
};
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
            generate_pdf_report(&path, results)?;
            println!("✅ PDF report written to {}", path.display());
        }
        other => bail!("Unsupported report format: {}", other),
    }

    Ok(())
}

fn generate_pdf_report(path: &Path, results: &[MatchResult]) -> Result<(), anyhow::Error> {
    let default_font = genpdf::fonts::from_files("./fonts", "Helvetica", Some(Builtin::Helvetica))
        .expect("Failed to load default font");

    let mut doc = Document::new(default_font);
    doc.set_title("Tooka File Sorting Report");
    doc.set_minimal_conformance();

    let title_style = style::Style::new().bold().with_font_size(24);

    // Add title
    let title = Paragraph::new("Tooka File Sorting Report").styled(title_style);

    // Add logo
    let image = Image::from_path("./assets/logo.png")
        .expect("Failed to load logo image")
        .with_alignment(Alignment::Center);

    // Add table
    let mut table = TableLayout::new(vec![1, 2, 3, 3]);

    table
        .row()
        .element(Paragraph::new("File"))
        .element(Paragraph::new("Rule"))
        .element(Paragraph::new("Current Path"))
        .element(Paragraph::new("New Path"))
        .push()?;

    for r in results {
        table
            .row()
            .element(Paragraph::new(&r.file_name))
            .element(Paragraph::new(&r.matched_rule_id))
            .element(Paragraph::new(r.current_path.display().to_string()))
            .element(Paragraph::new(r.new_path.display().to_string()))
            .push()?;
    }

    doc.push(title);
    doc.push(Break::new(1));
    doc.push(image);
    doc.push(Break::new(1));

    doc.push(table);
    doc.render_to_file(path)?;

    Ok(())
}
