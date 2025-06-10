use chrono::Local;
use printpdf::*;
use std::{collections::BTreeMap, fs::write, path::Path};

use crate::{core::context, core::sorter::MatchResult};

const PAGE_WIDTH: f32 = 210.0;
const PAGE_HEIGHT: f32 = 297.0;
const FONT_SIZE: f32 = 12.0;
const TITLE_FONT_SIZE: f32 = 20.0;
const MARGIN_X: f32 = 10.0;
const MARGIN_TOP: f32 = 20.0;
const LOGO_SCALE: f32 = 0.2;
const LOGO_POS_X: f32 = 5.0;
const LOGO_POS_Y: f32 = PAGE_HEIGHT - 15.0;
const TITLE_POS_X: f32 = 18.0;
const TITLE_POS_Y: f32 = PAGE_HEIGHT - 13.0;
const MAX_PATH_LENGTH: f32 = PAGE_WIDTH - 2.0 * MARGIN_X - 105.0;

pub(crate) fn generate_pdf(path: &Path, results: &[MatchResult]) -> Result<(), anyhow::Error> {
    let mut doc = PdfDocument::new("Tooka Report");
    let logo_svg =
        Svg::parse(context::LOGO_VECTOR_STR, &mut Vec::new()).map_err(anyhow::Error::msg)?;
    let logo_id = doc.add_xobject(&logo_svg);

    // Group results by rule
    let mut grouped: BTreeMap<String, Vec<&MatchResult>> = BTreeMap::new();
    for result in results {
        grouped
            .entry(result.matched_rule_id.clone())
            .or_default()
            .push(result);
    }

    let mut pages = Vec::new();
    let mut page_number = 1;

    // Chunk per page: ~6 match results per page
    let mut ops = Vec::new();
    let mut y = PAGE_HEIGHT - MARGIN_TOP - 20.0;

    ops.push(Op::SaveGraphicsState);
    draw_header(&mut ops, &logo_id, results.len())?;

    for (rule_id, entries) in grouped {
        // Section header
        write_text(
            &mut ops,
            &format!("> Rule: {}", rule_id),
            FONT_SIZE + 2.0,
            MARGIN_X,
            y,
            BuiltinFont::HelveticaBold,
        );
        y -= 10.0;

        for entry in entries {
            if (page_number == 1 && y < 50.0) || (page_number > 1 && y < 40.0) {
                // Close current page
                draw_footer(&mut ops, page_number);
                ops.push(Op::RestoreGraphicsState);
                pages.push(PdfPage::new(Mm(PAGE_WIDTH), Mm(PAGE_HEIGHT), ops));
                page_number += 1;

                // Start new page
                ops = Vec::new();
                y = PAGE_HEIGHT - 15.0;
                ops.push(Op::SaveGraphicsState);
                write_text(
                    &mut ops,
                    &format!("> Rule: {}", rule_id),
                    FONT_SIZE + 2.0,
                    MARGIN_X,
                    y,
                    BuiltinFont::HelveticaBold,
                );
                y -= 10.0;
            }

            draw_colored_box(
                &mut ops,
                MARGIN_X,
                y - 20.0,
                PAGE_WIDTH - 2.0 * MARGIN_X,
                25.0,
                Color::Greyscale(Greyscale {
                    percent: 0.95,
                    icc_profile: None,
                }),
            );
            draw_match_result_block(&mut ops, entry, y + 5.0);
            y -= 30.0;
        }

        y -= 5.0; // space after group
    }

    draw_footer(&mut ops, page_number);
    ops.push(Op::RestoreGraphicsState);
    pages.push(PdfPage::new(Mm(PAGE_WIDTH), Mm(PAGE_HEIGHT), ops));

    let pdf_bytes = doc
        .with_pages(pages)
        .save(&PdfSaveOptions::default(), &mut Vec::new());
    write(path, &pdf_bytes)?;
    Ok(())
}

fn draw_header(
    ops: &mut Vec<Op>,
    logo_id: &XObjectId,
    total_changes: usize,
) -> Result<(), anyhow::Error> {
    ops.push(Op::UseXobject {
        id: logo_id.clone(),
        transform: XObjectTransform {
            translate_x: Some(Mm(LOGO_POS_X).into_pt()),
            translate_y: Some(Mm(LOGO_POS_Y).into_pt()),
            scale_x: Some(LOGO_SCALE),
            scale_y: Some(LOGO_SCALE),
            ..Default::default()
        },
    });

    write_text(
        ops,
        "Tooka Report",
        TITLE_FONT_SIZE,
        TITLE_POS_X,
        TITLE_POS_Y,
        BuiltinFont::HelveticaBold,
    );

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    write_text(
        ops,
        &timestamp,
        FONT_SIZE - 1.0,
        PAGE_WIDTH - 45.0,
        PAGE_HEIGHT - 12.0,
        BuiltinFont::Helvetica,
    );

    write_text(
        ops,
        &format!("Total changes: {}", total_changes),
        FONT_SIZE,
        LOGO_POS_X,
        PAGE_HEIGHT - 25.0,
        BuiltinFont::Helvetica,
    );

    Ok(())
}

fn draw_footer(ops: &mut Vec<Op>, page_num: usize) {
    write_text(
        ops,
        &format!("Page {}", page_num),
        FONT_SIZE - 1.0,
        PAGE_WIDTH / 2.0 - 5.0,
        8.0,
        BuiltinFont::Helvetica,
    );
}

fn draw_match_result_block(ops: &mut Vec<Op>, r: &MatchResult, y_top: f32) {
    let font = BuiltinFont::Helvetica;
    let from_path = truncate_path(&r.current_path, MAX_PATH_LENGTH);
    let to_path = truncate_path(&r.new_path, MAX_PATH_LENGTH);

    write_text(
        ops,
        &format!("[{}] - {}", r.action, r.file_name),
        FONT_SIZE + 0.5,
        MARGIN_X + 2.0,
        y_top - 5.0,
        font,
    );
    write_text(
        ops,
        &format!("From: {}", from_path),
        FONT_SIZE,
        MARGIN_X + 4.0,
        y_top - 13.0,
        font,
    );
    write_text(
        ops,
        &format!("To:   {}", to_path),
        FONT_SIZE,
        MARGIN_X + 4.0,
        y_top - 20.0,
        font,
    );
}

fn draw_colored_box(ops: &mut Vec<Op>, x: f32, y: f32, width: f32, height: f32, color: Color) {
    let points = vec![
        LinePoint {
            p: Point::new(Mm(x), Mm(y)),
            bezier: false,
        },
        LinePoint {
            p: Point::new(Mm(x + width), Mm(y)),
            bezier: false,
        },
        LinePoint {
            p: Point::new(Mm(x + width), Mm(y + height)),
            bezier: false,
        },
        LinePoint {
            p: Point::new(Mm(x), Mm(y + height)),
            bezier: false,
        },
        LinePoint {
            p: Point::new(Mm(x), Mm(y)),
            bezier: false,
        }, // Closing the path
    ];

    let ring = PolygonRing { points };

    let polygon = Polygon {
        rings: vec![ring],
        mode: PaintMode::Fill,
        winding_order: WindingOrder::EvenOdd,
    };

    ops.push(Op::SetFillColor { col: color });
    ops.push(Op::DrawPolygon { polygon });
    ops.push(Op::SetFillColor {
        col: Color::Greyscale(Greyscale {
            percent: 0.0,
            icc_profile: None,
        }),
    }); // Reset fill color
}

fn write_text(ops: &mut Vec<Op>, text: &str, size: f32, x: f32, y: f32, font: BuiltinFont) {
    ops.push(Op::StartTextSection);
    ops.push(Op::SetTextCursor {
        pos: Point::new(Mm(x), Mm(y)),
    });
    ops.push(Op::SetFontSizeBuiltinFont {
        size: Pt(size),
        font,
    });
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text(text.to_string())],
        font,
    });
    ops.push(Op::EndTextSection);
}

fn truncate_path(path: &Path, max_len: f32) -> String {
    let full = path.display().to_string();
    if full.len() <= max_len as usize {
        return full;
    }

    // Try to get parent and filename
    if let (Some(parent), Some(file_name)) = (path.parent(), path.file_name()) {
        let parent_str = parent
            .file_name()
            .or(Some(parent.as_os_str()))
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| parent.display().to_string());

        return format!(
            "[truncated].../{}/{}",
            parent_str,
            file_name.to_string_lossy()
        );
    }

    // Fallback: just truncate the string with ellipsis
    if full.len() > max_len as usize {
        format!("{}...", &full[..max_len as usize - 3])
    } else {
        full
    }
}
