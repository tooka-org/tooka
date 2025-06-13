use crate::sorter::MatchResult;
use chrono::Local;
use pdf_writer::{Chunk, Content, Name, Pdf, Rect, Ref, Str};
use std::{collections::BTreeMap, path::Path};

const PAGE_WIDTH: f32 = 595.0;
const PAGE_HEIGHT: f32 = 842.0;
const FONT_SIZE: f32 = 12.0;
const TITLE_FONT_SIZE: f32 = 20.0;
const MARGIN_X: f32 = 20.0;
const MARGIN_TOP: f32 = 40.0;
const TITLE_POS_X: f32 = MARGIN_X;
const TITLE_POS_Y: f32 = PAGE_HEIGHT - MARGIN_TOP;
const TIME_POS_X: f32 = PAGE_WIDTH - 110.0;
const TIME_POS_Y: f32 = PAGE_HEIGHT - 15.0;
const MAX_PATH_LENGTH: f32 = PAGE_WIDTH - 2.0 * MARGIN_X - 105.0;

pub(crate) fn generate_pdf(path: &Path, results: &[MatchResult]) -> Result<(), anyhow::Error> {
    let mut alloc = Ref::new(1);
    let mut pdf = Pdf::new();

    let font_id = alloc.bump();
    pdf.type1_font(font_id).base_font(Name(b"Helvetica"));
    let font_name = Name(b"Helvetica");

    let mut secondary = Chunk::new();
    let page_tree_id = alloc.bump();
    let mut page_ids = vec![];

    // Group results by rule and sort each group by file name
    let mut grouped: BTreeMap<String, Vec<&MatchResult>> = BTreeMap::new();
    for result in results {
        grouped
            .entry(result.matched_rule_id.clone())
            .or_default()
            .push(result);
    }

    // Sort entries by file_name within each rule_id group
    for entries in grouped.values_mut() {
        entries.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    }

    // Flatten into display order: sorted by rule_id (already sorted in BTreeMap), then file_name
    let mut flat_entries = vec![];
    for (rule_id, entries) in grouped {
        flat_entries.push((Some(rule_id), None)); // Section header marker
        for entry in entries {
            flat_entries.push((None, Some(entry)));
        }
    }

    let mut page_number = 1;
    let mut y = PAGE_HEIGHT - MARGIN_TOP - 50.0;
    let min_y = MARGIN_TOP + 40.0;
    let mut content = Content::new();
    let mut extg_states = vec![];
    let mut first_page = true;

    let mut last_rule_id: Option<String> = None;

    for (rule_id_opt, entry_opt) in flat_entries {
        // Section header
        if let Some(rule_id) = rule_id_opt {
            // If not enough space for section header, start new page
            if y < min_y + 30.0 {
                // Finish current page
                let page_id = alloc.bump();
                page_ids.push(page_id);
                let mut page = pdf.page(page_id);
                page.media_box(Rect::new(0.0, 0.0, PAGE_WIDTH, PAGE_HEIGHT));
                page.parent(page_tree_id);

                if first_page {
                    draw_header(&mut content, results.len(), font_name);
                    first_page = false;
                }
                draw_footer(&mut content, page_number, font_name);

                let content_id = alloc.bump();
                secondary.stream(content_id, &content.finish());
                page.contents(content_id);
                page.resources().ext_g_states().pairs(
                    extg_states
                        .iter()
                        .map(|(n, id): &(String, Ref)| (Name(n.as_bytes()), *id)),
                );

                page_number += 1;
                y = PAGE_HEIGHT - MARGIN_TOP - 50.0;
                content = Content::new();
                extg_states = vec![];
            }

            write_text(
                &mut content,
                &format!("> Rule: {}", rule_id),
                FONT_SIZE + 2.0,
                MARGIN_X,
                y,
                font_name,
            );
            y -= 30.0;
            last_rule_id = Some(rule_id);
        }

        // Entry
        if let Some(entry) = entry_opt {
            // If not enough space for entry, start new page
            if y < min_y + 50.0 {
                // Finish current page
                let page_id = alloc.bump();
                page_ids.push(page_id);
                let mut page = pdf.page(page_id);
                page.media_box(Rect::new(0.0, 0.0, PAGE_WIDTH, PAGE_HEIGHT));
                page.parent(page_tree_id);

                if first_page {
                    draw_header(&mut content, results.len(), font_name);
                    first_page = false;
                }
                draw_footer(&mut content, page_number, font_name);

                let content_id = alloc.bump();
                secondary.stream(content_id, &content.finish());
                page.contents(content_id);
                page.resources()
                    .ext_g_states()
                    .pairs(extg_states.iter().map(|(n, id)| (Name(n.as_bytes()), id)));

                page_number += 1;
                y = PAGE_HEIGHT - MARGIN_TOP - 50.0;
                content = Content::new();
                extg_states = vec![];

                // Optionally, repeat section header at top of new page
                if let Some(rule_id) = &last_rule_id {
                    write_text(
                        &mut content,
                        &format!("> Rule: {}", rule_id),
                        FONT_SIZE + 2.0,
                        MARGIN_X,
                        y,
                        font_name,
                    );
                    y -= 30.0;
                }
            }

            let state_name = format!("G_{}_{}", entry.matched_rule_id, entry.file_name);
            content.set_parameters(Name(state_name.as_bytes()));
            draw_colored_box(
                &mut content,
                MARGIN_X,
                y - 30.0,
                PAGE_WIDTH - 2.0 * MARGIN_X,
                50.0,
                (0.9, 0.9, 0.9),
            );
            draw_match_result_block(&mut content, entry, y + 5.0, font_name);
            y -= 60.0;
            let state_id = alloc.bump();
            extg_states.push((state_name, state_id));
        }
    }

    // Write the last page if it has content
    if !extg_states.is_empty() {
        let page_id = alloc.bump();
        page_ids.push(page_id);
        let mut page = pdf.page(page_id);
        page.media_box(Rect::new(0.0, 0.0, PAGE_WIDTH, PAGE_HEIGHT));
        page.parent(page_tree_id);

        if first_page {
            draw_header(&mut content, results.len(), font_name);
        }
        draw_footer(&mut content, page_number, font_name);

        let content_id = alloc.bump();
        secondary.stream(content_id, &content.finish());
        page.contents(content_id);
        page.resources()
            .ext_g_states()
            .pairs(extg_states.iter().map(|(n, id)| (Name(n.as_bytes()), id)));
    }

    pdf.extend(&secondary);

    pdf.pages(page_tree_id)
        .kids(page_ids.iter().copied())
        .count(page_ids.len() as i32);

    pdf.catalog(alloc.bump()).pages(page_tree_id);

    std::fs::write(path, pdf.finish())?;
    Ok(())
}

fn write_text(content: &mut Content, text: &str, font_size: f32, x: f32, y: f32, font_name: Name) {
    content.begin_text();
    content.next_line(x, y);
    content.set_font(font_name, font_size);
    content.show(Str(text.as_bytes()));
    content.end_text();
}

fn draw_footer(content: &mut Content, page_num: usize, font_name: Name) {
    write_text(
        content,
        &format!("Page {}", page_num),
        FONT_SIZE - 1.0,
        PAGE_WIDTH / 2.0 - 20.0,
        8.0,
        font_name,
    );
}

fn draw_header(content: &mut Content, total_changes: usize, font_name: Name) {
    write_text(
        content,
        "Tooka Report",
        TITLE_FONT_SIZE,
        TITLE_POS_X,
        TITLE_POS_Y,
        font_name,
    );

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    write_text(
        content,
        &timestamp,
        FONT_SIZE - 1.0,
        TIME_POS_X,
        TIME_POS_Y,
        font_name,
    );

    write_text(
        content,
        &format!("Total changes: {}", total_changes),
        FONT_SIZE,
        TITLE_POS_X,
        TITLE_POS_Y - 15.0,
        font_name,
    );
}

fn draw_match_result_block(
    content: &mut Content,
    result: &MatchResult,
    y_top: f32,
    font_name: Name,
) {
    let from_path = truncate_path(&result.current_path, MAX_PATH_LENGTH);
    let to_path = truncate_path(&result.new_path, MAX_PATH_LENGTH);

    // Set colors based on action
    let color = match result.action.as_str() {
        "move" => (0.2, 0.4, 0.8),    // Blue-ish
        "copy" => (0.2, 0.7, 0.3),    // Green-ish
        "delete" => (0.85, 0.3, 0.3), // Red-ish
        "rename" => (0.8, 0.6, 0.2),  // Orange-ish
        "execute" => (0.5, 0.2, 0.7), // Purple-ish
        "skip" => (0.6, 0.6, 0.6),    // Grey
        _ => (0.0, 0.0, 0.0),         // Default to black
    };

    content.set_fill_rgb(color.0, color.1, color.2);

    write_text(
        content,
        &format!("[{}] - {}", result.action, result.file_name),
        FONT_SIZE + 0.5,
        MARGIN_X + 10.0,
        y_top,
        font_name,
    );

    content.set_fill_rgb(0.0, 0.0, 0.0); // Reset fill color to black

    write_text(
        content,
        &format!("From: {}", from_path),
        FONT_SIZE,
        MARGIN_X + 15.0,
        y_top - 15.0,
        font_name,
    );
    write_text(
        content,
        &format!("To:     {}", to_path),
        FONT_SIZE,
        MARGIN_X + 15.0,
        y_top - 30.0,
        font_name,
    );
}

fn draw_colored_box(
    content: &mut Content,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: (f32, f32, f32),
) {
    content.set_fill_rgb(color.0, color.1, color.2);
    content.rect(x, y, width, height);
    content.fill_even_odd();
    content.set_fill_rgb(0.0, 0.0, 0.0); // Reset fill color to black
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
