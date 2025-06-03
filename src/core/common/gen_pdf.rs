use printpdf::{PdfSaveOptions, Pt, TextItem};
use std::path::Path;

use crate::{context, core::sorter::MatchResult};

// Try not to change, might break the layout
const PAGE_WIDTH: f32 = 210.0;
const PAGE_HEIGHT: f32 = 297.0;
const FONT_SIZE: f32 = 15.0;
const LINE_HEIGHT: f32 = FONT_SIZE + 6.0;
const LINE_PADDING: f32 = 3.0;
const LOGO_SCALE: f32 = 0.2;
const LOGO_POS_X: f32 = 5.0;
const LOGO_POS_Y: f32 = PAGE_HEIGHT - 15.0;
const TITLE_FONT_SIZE: f32 = 20.0;
const TITLE_POS_X: f32 = 18.0;
const TITLE_POS_Y: f32 = LOGO_POS_Y + 2.0;
const PAGE_NUMBER_FONT_SIZE: f32 = FONT_SIZE - 2.0;
const DATE_FONT_SIZE: f32 = 12.0;
const DATE_POS_X: f32 = PAGE_WIDTH - 50.0;
const DATE_POS_Y: f32 = PAGE_HEIGHT - 10.0;
const PAGE_NUMBER_POS_X: f32 = PAGE_WIDTH / 2.0 - 2.0;
const PAGE_NUMBER_POS_Y: f32 = 5.0;
// For the first page only
const FIRST_CONTENT_POS_X: f32 = LOGO_POS_X;
const FIRST_CONTENT_POS_Y: f32 = PAGE_HEIGHT - 35.0;
// For the rest of the pages
const CONTENT_POS_X: f32 = LOGO_POS_X;
const CONTENT_POS_Y: f32 = PAGE_HEIGHT - 10.0;

// NOTE: P(0,0) is bottom left corner
// NOTE: P(x,y) = P(width, height) is top right corner

pub fn generate_pdf(
    path: &Path,
    results: &[MatchResult]
) -> Result<(), anyhow::Error> {
    let mut doc = printpdf::PdfDocument::new("Tooka Report");
    let mut page_ops = Vec::new();
    let mut pages = Vec::new();

    let svg = printpdf::Svg::parse(context::LOGO_VECTOR_STR, &mut Vec::new())
        .map_err(|err| format!("Failed to load logo: {err}"))
        .unwrap();

    // >> START HEADER
    let svg_id = doc.add_xobject(&svg);

    page_ops.push(printpdf::Op::SaveGraphicsState);

    page_ops.push(printpdf::Op::UseXobject {
        id: svg_id.clone(),
        transform: printpdf::XObjectTransform {
            translate_x: Some(printpdf::Mm(LOGO_POS_X).into_pt()),
            translate_y: Some(printpdf::Mm(LOGO_POS_Y).into_pt()),
            scale_x: Some(LOGO_SCALE),
            scale_y: Some(LOGO_SCALE),
            ..Default::default()
        },
    });

    page_ops.push(printpdf::Op::StartTextSection);
    page_ops.push(printpdf::Op::SetTextCursor {
        pos: printpdf::Point::new(printpdf::Mm(TITLE_POS_X), printpdf::Mm(TITLE_POS_Y)),
    });
    page_ops.push(printpdf::Op::SetFontSizeBuiltinFont {
        size: Pt(TITLE_FONT_SIZE),
        font: printpdf::BuiltinFont::Helvetica,
    });
    page_ops.push(printpdf::Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text("Tooka Report".to_string())],
        font: printpdf::BuiltinFont::Helvetica,
    });

    // Create date string from file timestamp or set to default
    let now = chrono::Local::now();
    let now_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

    page_ops.push(printpdf::Op::SetTextCursor {
        pos: printpdf::Point::new(printpdf::Mm(DATE_POS_X), printpdf::Mm(DATE_POS_Y)),
    });
    page_ops.push(printpdf::Op::SetFontSizeBuiltinFont {
        size: Pt(DATE_FONT_SIZE),
        font: printpdf::BuiltinFont::Helvetica,
    });
    page_ops.push(printpdf::Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text(now_str)],
        font: printpdf::BuiltinFont::Helvetica,
    });

    page_ops.push(printpdf::Op::EndTextSection);

    // Add a line below the title (Set the start and end points)
    let line_start_point = printpdf::Point::new(
        printpdf::Mm(LOGO_POS_X - 2.0),
        printpdf::Mm(LOGO_POS_Y - LINE_PADDING),
    );

    let line_end_point = printpdf::Point::new(
        printpdf::Mm(PAGE_WIDTH - LINE_PADDING),
        printpdf::Mm(LOGO_POS_Y - LINE_PADDING),
    );

    page_ops.push(printpdf::Op::DrawLine {
        line: printpdf::Line {
            points: vec![
                printpdf::LinePoint {
                    p: line_start_point,
                    bezier: true,
                },
                printpdf::LinePoint {
                    p: line_end_point,
                    bezier: true,
                },
            ],
            is_closed: true,
        },
    });

    page_ops.push(printpdf::Op::StartTextSection);
    page_ops.push(printpdf::Op::SetTextCursor {
        pos: printpdf::Point::new(printpdf::Mm(LOGO_POS_X), printpdf::Mm(LOGO_POS_Y - 10.0)),
    });
    page_ops.push(printpdf::Op::SetFontSizeBuiltinFont {
        size: Pt(12.0),
        font: printpdf::BuiltinFont::Helvetica,
    });
    page_ops.push(printpdf::Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text(
            format!("Total changes: {}", results.len()).to_string(),
        )],
        font: printpdf::BuiltinFont::Helvetica,
    });
    page_ops.push(printpdf::Op::EndTextSection);

    // << HEADER IS DONE

    // Add content (VERY SIMILAR TO OLD CODE)
    // weird code with magic numbers but what can you do /shrug
    let pt_in_mm = (LINE_HEIGHT) * 0.3537778;
    let max_lines_page = (FIRST_CONTENT_POS_Y / pt_in_mm) as usize;
    let max_chars_per_line = (PAGE_WIDTH - 10.0) / (FONT_SIZE * 0.5 * 0.3537778);
    log::debug!(
        "Determined lines per page should be {max_lines_page}, and max chars per line should be {max_chars_per_line}"
    );

    // Start of the first page
    page_ops.push(printpdf::Op::StartTextSection);
    page_ops.push(printpdf::Op::SetTextCursor {
        pos: printpdf::Point::new(
            printpdf::Mm(FIRST_CONTENT_POS_X),
            printpdf::Mm(FIRST_CONTENT_POS_Y),
        ),
    });
    page_ops.push(printpdf::Op::SetFontSizeBuiltinFont {
        size: Pt(FONT_SIZE),
        font: printpdf::BuiltinFont::Helvetica,
    });
    page_ops.push(printpdf::Op::SetLineHeight {
        lh: Pt(LINE_HEIGHT),
    });

    // Counter for the current line and page
    let mut current_line = 1;
    let mut current_page = 1;

    let mut result_lines = Vec::new();
    // add changed files to the total lines
    for changed_file in results {
        // add initial 'header' for a particular changed file
        result_lines.push(format!(
            "> [{}] {}",
            changed_file.file_name, changed_file.matched_rule_id
        ));

        result_lines.push(
            format!(
                "\t {} -> {}",
                changed_file.current_path.display(),
                changed_file.new_path.display()
            ),
        );
    }

    for line in result_lines {
        // if we reach the maximum lines per page we create a new one
        if current_line % max_lines_page == 0 && current_line > 0 {
            current_page += 1;
            // cleanup old page
            page_ops.push(printpdf::Op::EndTextSection);
            pages.push(printpdf::PdfPage::new(
                printpdf::Mm(PAGE_WIDTH),
                printpdf::Mm(PAGE_HEIGHT),
                page_ops,
            ));
            page_ops = vec![
                printpdf::Op::RestoreGraphicsState,
                printpdf::Op::StartTextSection,
                printpdf::Op::SetTextCursor {
                    pos: printpdf::Point::new(
                        printpdf::Mm(CONTENT_POS_X),
                        printpdf::Mm(CONTENT_POS_Y),
                    ),
                },
                printpdf::Op::SetFontSizeBuiltinFont {
                    size: Pt(FONT_SIZE),
                    font: printpdf::BuiltinFont::Helvetica,
                },
                printpdf::Op::SetLineHeight {
                    lh: Pt(LINE_HEIGHT),
                },
            ];
        }

        // write the current line to pdf
        let mut lines = Vec::new();

        // split line into pieces that fit on the page
        lines.extend(
            line.chars()
                .collect::<Vec<char>>()
                .chunks(max_chars_per_line as usize)
                .map(|chars| chars.iter().collect())
                .collect::<Vec<String>>(),
        );

        for line in lines {
            page_ops.push(printpdf::Op::WriteTextBuiltinFont {
                items: vec![TextItem::Text(line.to_string())],
                font: printpdf::BuiltinFont::Helvetica,
            });
            page_ops.push(printpdf::Op::AddLineBreak);

            current_line += 1;
        }
    }

    page_ops.push(printpdf::Op::SetTextCursor {
        pos: printpdf::Point::new(
            printpdf::Mm(PAGE_NUMBER_POS_X),
            printpdf::Mm(PAGE_NUMBER_POS_Y),
        ),
    });
    page_ops.push(printpdf::Op::SetFontSizeBuiltinFont {
        size: Pt(PAGE_NUMBER_FONT_SIZE),
        font: printpdf::BuiltinFont::Helvetica,
    });
    page_ops.push(printpdf::Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text(current_page.to_string())],
        font: printpdf::BuiltinFont::Helvetica,
    });
    page_ops.push(printpdf::Op::EndTextSection);

    // Add the last page operations to the pages vector
    pages.push(printpdf::PdfPage::new(
        printpdf::Mm(PAGE_WIDTH),
        printpdf::Mm(PAGE_HEIGHT),
        page_ops,
    ));

    // Save the PDF to a file
    let pdf_bytes = doc
        .with_pages(pages)
        .save(&PdfSaveOptions::default(), &mut Vec::new());
    std::fs::write(path, &pdf_bytes)
        .map_err(|err| anyhow::anyhow!("Failed to save pdf to file: {err}"))?;

    Ok(())
}
