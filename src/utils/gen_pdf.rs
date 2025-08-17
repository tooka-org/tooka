use crate::core::sorter::MatchResult;
use chrono::Local;
use pdf_writer::{Chunk, Content, Name, Pdf, Rect, Ref, Str};
use std::{collections::BTreeMap, path::Path};

// Page dimensions and basic layout
const PAGE_WIDTH: f32 = 595.0;
const PAGE_HEIGHT: f32 = 842.0;
const MARGIN_X: f32 = 20.0;
const MARGIN_TOP: f32 = 40.0;

// Font sizes
const FONT_SIZE: f32 = 12.0;
const TITLE_FONT_SIZE: f32 = 20.0;
const FOOTER_FONT_SIZE: f32 = FONT_SIZE - 1.0;
const TIMESTAMP_FONT_SIZE: f32 = FONT_SIZE - 1.0;
const ACTION_FONT_SIZE: f32 = FONT_SIZE + 0.5;

// Header and footer positioning
const TITLE_POS_X: f32 = MARGIN_X;
const TITLE_POS_Y: f32 = PAGE_HEIGHT - MARGIN_TOP;
const TIME_POS_X: f32 = PAGE_WIDTH - 110.0;
const TIME_POS_Y: f32 = PAGE_HEIGHT - 15.0;
const FOOTER_Y: f32 = 8.0;
const PAGE_NUMBER_X_OFFSET: f32 = 20.0;

// Content layout constants
const CONTENT_START_OFFSET: f32 = 50.0;
const MIN_Y_OFFSET: f32 = 40.0;
const RULE_BEFORE_SPACING: f32 = 15.0; // Space before rule title when following content
const RULE_AFTER_SPACING: f32 = 20.0;  // Space after rule title before next content
const RULE_FONT_SIZE_OFFSET: f32 = 2.0;
const TOTAL_CHANGES_Y_OFFSET: f32 = 15.0;

// Box and content styling
const BOX_PADDING: f32 = 20.0;
const BOX_TOP_PADDING: f32 = 20.0;
const BOX_BOTTOM_SPACING: f32 = 10.0;
const BOX_CORNER_RADIUS: f32 = 4.0;
const BEZIER_CONTROL_RATIO: f32 = 0.552; // Ratio for circular arc approximation with cubic bezier
const CONTENT_INDENT: f32 = 10.0;
const TEXT_SECTION_SPACING: f32 = 20.0;
const PATH_SECTION_SPACING: f32 = 15.0;
const LINE_HEIGHT: f32 = 12.0;
const CONTENT_BASE_HEIGHT: f32 = 30.0;

// Text positioning and wrapping
const MAX_PATH_LENGTH: f32 = PAGE_WIDTH - 2.0 * MARGIN_X - 105.0;
const APPROX_CHAR_WIDTH: f32 = 6.0;
const FROM_TO_INDENT: f32 = 15.0;
const PATH_VALUE_INDENT: f32 = 50.0;
const PATH_CONTINUATION_INDENT: f32 = 35.0;
const MAX_PATH_LINES: usize = 3;

/// PDF generator that manages state and rendering for creating reports
struct PDFGenerator {
    pdf: Pdf,
    alloc: Ref,
    page_tree_id: Ref,
    font_name: Name<'static>,
    font_id: Ref,
    page_ids: Vec<Ref>,
    content: Content,
    extg_states: Vec<(String, Ref)>,
    page_number: i32,
    y: f32,
    first_page: bool,
    last_rule_id: Option<String>,
    total_results: usize,
}

impl PDFGenerator {
    fn new(total_results: usize) -> Self {
        let mut alloc = Ref::new(1);
        let mut pdf = Pdf::new();
        
        let (font_name, font_id) = Self::init_fonts(&mut pdf, &mut alloc);
        let page_tree_id = alloc.bump();
        
        Self {
            pdf,
            alloc,
            page_tree_id,
            font_name,
            font_id,
            page_ids: vec![],
            content: Content::new(),
            extg_states: vec![],
            page_number: 1,
            y: PAGE_HEIGHT - MARGIN_TOP - CONTENT_START_OFFSET,
            first_page: true,
            last_rule_id: None,
            total_results,
        }
    }
    
    fn generate(mut self, path: &Path, results: &[MatchResult]) -> Result<(), anyhow::Error> {
        let flat_entries = Self::prepare_entries(results);
        self.render_pages(&flat_entries);
        self.finalize();
        
        std::fs::write(path, self.pdf.finish())?;
        Ok(())
    }

    fn init_fonts(pdf: &mut Pdf, alloc: &mut Ref) -> (Name<'static>, Ref) {
        let font_id = alloc.bump();
        pdf.type1_font(font_id).base_font(Name(b"Helvetica"));
        (Name(b"Helvetica"), font_id)
    }

    fn prepare_entries(results: &[MatchResult]) -> Vec<(Option<String>, Option<&MatchResult>)> {
        let mut grouped: BTreeMap<String, Vec<&MatchResult>> = BTreeMap::new();
        for result in results {
            grouped
                .entry(result.matched_rule_id.clone())
                .or_default()
                .push(result);
        }
        for entries in grouped.values_mut() {
            entries.sort_by(|a, b| a.file_name.cmp(&b.file_name));
        }

        let mut flat_entries = vec![];
        for (rule_id, entries) in grouped {
            flat_entries.push((Some(rule_id), None));
            for entry in entries {
                flat_entries.push((None, Some(entry)));
            }
        }
        flat_entries
    }

    fn render_pages(&mut self, flat_entries: &[(Option<String>, Option<&MatchResult>)]) {
        let secondary = Chunk::new();
        let min_y = MARGIN_TOP + MIN_Y_OFFSET;
        let mut first_rule = true;

        for (rule_id_opt, entry_opt) in flat_entries {
            if let Some(rule_id) = rule_id_opt {
                // Add spacing before rule title if it's not the first rule and not at top of page
                if !first_rule && self.y < PAGE_HEIGHT - MARGIN_TOP - CONTENT_START_OFFSET {
                    self.y -= RULE_BEFORE_SPACING;
                }
                
                self.page_break_if_needed(min_y + RULE_AFTER_SPACING);

                self.write_text(
                    &format!("> Rule: {rule_id}"),
                    FONT_SIZE + RULE_FONT_SIZE_OFFSET,
                    MARGIN_X,
                    self.y,
                );
                self.y -= RULE_AFTER_SPACING; // Reduced spacing after rule title
                self.last_rule_id = Some(rule_id.clone());
                first_rule = false;
            }

            if let Some(entry) = entry_opt {
                // Calculate needed space for this entry (considering path wrapping)
                let from_lines = PDFGenerator::format_path_with_wrapping(&entry.current_path, MAX_PATH_LENGTH);
                let to_lines = PDFGenerator::format_path_with_wrapping(&entry.new_path, MAX_PATH_LENGTH);
                let total_lines = from_lines.len() + to_lines.len();
                let content_height = CONTENT_BASE_HEIGHT + (total_lines as f32 * LINE_HEIGHT); // Header + path lines
                let box_height = content_height + BOX_PADDING; // Add padding
                
                self.page_break_if_needed(min_y + box_height);

                if self.y == PAGE_HEIGHT - MARGIN_TOP - CONTENT_START_OFFSET {
                    if let Some(rule_id) = &self.last_rule_id {
                        self.write_text(
                            &format!("> Rule: {rule_id}"),
                            FONT_SIZE + RULE_FONT_SIZE_OFFSET,
                            MARGIN_X,
                            self.y,
                        );
                        self.y -= RULE_AFTER_SPACING; // Use consistent spacing after rule
                    }
                }

                let state_name = format!("G_{}_{}", entry.matched_rule_id, entry.file_name);
                self.content.set_parameters(Name(state_name.as_bytes()));
                
                // Calculate box position (box goes from bottom to top in PDF coordinates)
                let box_bottom = self.y - box_height;
                let box_top = self.y;
                
                // Draw the background box
                self.draw_colored_box(
                    MARGIN_X,
                    box_bottom,
                    PAGE_WIDTH - 2.0 * MARGIN_X,
                    box_height,
                    (0.9, 0.9, 0.9),
                );
                
                // Draw content with proper positioning (text starts from top of box with padding)
                let text_start_y = box_top - BOX_TOP_PADDING; // Start points from top of box
                self.draw_match_result_block(entry, text_start_y);
                
                self.y = box_bottom - BOX_BOTTOM_SPACING; // Move position down past the box plus spacing
                let state_id = self.alloc.bump();
                self.extg_states.push((state_name, state_id));
            }
        }

        if !self.extg_states.is_empty() {
            self.finish_page();
        }

        self.pdf.extend(&secondary);
    }

    fn page_break_if_needed(&mut self, min_y: f32) {
        if self.y < min_y {
            self.finish_page();
            self.page_number += 1;
            self.y = PAGE_HEIGHT - MARGIN_TOP - CONTENT_START_OFFSET;
            self.content = Content::new();
            self.extg_states = vec![];
            self.first_page = false;
        }
    }

    fn finish_page(&mut self) {
        let page_id = self.alloc.bump();
        self.page_ids.push(page_id);
        
        // Draw header and footer before creating the page
        if self.first_page {
            self.draw_header();
        }
        self.draw_footer();

        // Create the content stream and write it to the PDF
        let content_id = self.alloc.bump();
        let content_data = std::mem::replace(&mut self.content, Content::new()).finish();
        self.pdf.stream(content_id, &content_data);

        // Now create the page and set its properties
        let mut page = self.pdf.page(page_id);
        page.media_box(Rect::new(0.0, 0.0, PAGE_WIDTH, PAGE_HEIGHT));
        page.parent(self.page_tree_id);
        page.contents(content_id);
        
        // Add font resources
        let mut resources = page.resources();
        resources.fonts().pair(self.font_name, self.font_id);
        
        // Add graphics state resources if any
        if !self.extg_states.is_empty() {
            resources.ext_g_states().pairs(
                self.extg_states
                    .iter()
                    .map(|(n, id): &(String, Ref)| (Name(n.as_bytes()), *id)),
            );
        }
    }

    fn finalize(&mut self) {
        self.pdf.pages(self.page_tree_id)
            .kids(self.page_ids.iter().copied())
            .count(i32::try_from(self.page_ids.len()).unwrap_or(0));
        self.pdf.catalog(self.alloc.bump()).pages(self.page_tree_id);
    }

    fn write_text(&mut self, text: &str, font_size: f32, x: f32, y: f32) {
        self.content.begin_text();
        self.content.next_line(x, y);
        self.content.set_font(self.font_name, font_size);
        self.content.show(Str(text.as_bytes()));
        self.content.end_text();
    }

    fn draw_footer(&mut self) {
        self.write_text(
            &format!("Page {}", self.page_number),
            FOOTER_FONT_SIZE,
            PAGE_WIDTH / 2.0 - PAGE_NUMBER_X_OFFSET,
            FOOTER_Y,
        );
    }

    fn draw_header(&mut self) {
        self.write_text(
            "Tooka Report",
            TITLE_FONT_SIZE,
            TITLE_POS_X,
            TITLE_POS_Y,
        );

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.write_text(
            &timestamp,
            TIMESTAMP_FONT_SIZE,
            TIME_POS_X,
            TIME_POS_Y,
        );

        self.write_text(
            &format!("Total changes: {}", self.total_results),
            FONT_SIZE,
            TITLE_POS_X,
            TITLE_POS_Y - TOTAL_CHANGES_Y_OFFSET,
        );
    }

    fn draw_match_result_block(&mut self, result: &MatchResult, y_start: f32) {
        let from_path = PDFGenerator::format_path_with_wrapping(&result.current_path, MAX_PATH_LENGTH);
        let to_path = PDFGenerator::format_path_with_wrapping(&result.new_path, MAX_PATH_LENGTH);

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

        self.content.set_fill_rgb(color.0, color.1, color.2);

        // Start with action header at the top of the content area
        let mut current_y = y_start;
        self.write_text(
            &format!("[{}] - {}", result.action, result.file_name),
            ACTION_FONT_SIZE,
            MARGIN_X + CONTENT_INDENT,
            current_y,
        );

        self.content.set_fill_rgb(0.0, 0.0, 0.0); // Reset fill color to black

        // Move down for the paths section
        current_y -= TEXT_SECTION_SPACING;
        
        // Draw "From:" path
        self.write_text("From:", FONT_SIZE, MARGIN_X + FROM_TO_INDENT, current_y);
        for (i, line) in from_path.iter().enumerate() {
            let x_offset = if i == 0 { PATH_VALUE_INDENT } else { PATH_CONTINUATION_INDENT }; // Indent continuation lines
            self.write_text(line, FONT_SIZE, MARGIN_X + x_offset, current_y);
            if i < from_path.len() - 1 {
                current_y -= LINE_HEIGHT; // Move to next line for wrapped text
            }
        }
        
        current_y -= PATH_SECTION_SPACING; // Space between from and to
        
        // Draw "To:" path  
        self.write_text("To:", FONT_SIZE, MARGIN_X + FROM_TO_INDENT, current_y);
        for (i, line) in to_path.iter().enumerate() {
            let x_offset = if i == 0 { PATH_VALUE_INDENT } else { PATH_CONTINUATION_INDENT }; // Indent continuation lines (align with "To:")
            self.write_text(line, FONT_SIZE, MARGIN_X + x_offset, current_y);
            if i < to_path.len() - 1 {
                current_y -= LINE_HEIGHT; // Move to next line for wrapped text
            }
        }
    }

    /// Format a path with intelligent wrapping/truncation
    fn format_path_with_wrapping(path: &Path, max_width: f32) -> Vec<String> {
        let full_path = path.display().to_string();
        let approx_char_width = APPROX_CHAR_WIDTH; // Approximate character width in points for font size 12
        let max_chars_per_line = (max_width / approx_char_width) as usize;
        
        if full_path.len() <= max_chars_per_line {
            return vec![full_path];
        }

        // Try to break at path separators first
        let mut lines = Vec::new();
        let mut current_line = String::new();
        
        // Split by path separators and try to fit segments
        let parts: Vec<&str> = full_path.split('/').collect();
        
        for (i, part) in parts.iter().enumerate() {
            let separator = if i == 0 { "" } else { "/" };
            let potential_addition = format!("{separator}{part}");
            
            // If adding this part would exceed the line length
            if current_line.len() + potential_addition.len() > max_chars_per_line {
                // If current line is not empty, save it
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
                
                // If this single part is too long, truncate it
                if part.len() > max_chars_per_line {
                    let truncated = if part.len() > max_chars_per_line - 3 {
                        format!("{}...", &part[..max_chars_per_line - 3])
                    } else {
                        (*part).to_string()
                    };
                    lines.push(format!("{separator}{truncated}"));
                } else {
                    current_line = potential_addition;
                }
            } else {
                current_line.push_str(&potential_addition);
            }
        }
        
        // Add the last line if it's not empty
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        
        // Ensure we don't have too many lines (max lines for readability)
        if lines.len() > MAX_PATH_LINES {
            lines.truncate(MAX_PATH_LINES - 1);
            lines.push("... (path continues)".to_string());
        }
        
        // If still empty, return a fallback
        if lines.is_empty() {
            lines.push(truncate_path(path, max_width));
        }
        
        lines
    }

    fn draw_colored_box(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: (f32, f32, f32),
    ) {
        self.content.set_fill_rgb(color.0, color.1, color.2);
        
        // Draw rounded rectangle using path operations
        let radius = BOX_CORNER_RADIUS; // Corner radius
        
        // Start from top-left corner (after the radius)
        self.content.move_to(x + radius, y + height);
        
        // Top edge
        self.content.line_to(x + width - radius, y + height);
        // Top-right corner (using cubic bezier curve)
        self.content.cubic_to(
            x + width - radius + radius * BEZIER_CONTROL_RATIO, y + height,
            x + width, y + height - radius + radius * BEZIER_CONTROL_RATIO,
            x + width, y + height - radius,
        );
        
        // Right edge
        self.content.line_to(x + width, y + radius);
        // Bottom-right corner
        self.content.cubic_to(
            x + width, y + radius - radius * BEZIER_CONTROL_RATIO,
            x + width - radius + radius * BEZIER_CONTROL_RATIO, y,
            x + width - radius, y,
        );
        
        // Bottom edge
        self.content.line_to(x + radius, y);
        // Bottom-left corner
        self.content.cubic_to(
            x + radius - radius * BEZIER_CONTROL_RATIO, y,
            x, y + radius - radius * BEZIER_CONTROL_RATIO,
            x, y + radius,
        );
        
        // Left edge
        self.content.line_to(x, y + height - radius);
        // Top-left corner
        self.content.cubic_to(
            x, y + height - radius + radius * BEZIER_CONTROL_RATIO,
            x + radius - radius * BEZIER_CONTROL_RATIO, y + height,
            x + radius, y + height,
        );
        
        self.content.fill_even_odd();
        self.content.set_fill_rgb(0.0, 0.0, 0.0); // Reset fill color to black
    }
}

pub(crate) fn generate_pdf(path: &Path, results: &[MatchResult]) -> Result<(), anyhow::Error> {
    let generator = PDFGenerator::new(results.len());
    generator.generate(path, results)
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
            .map_or_else(|| parent.display().to_string(), |s| s.to_string_lossy().to_string());

        return format!(
            "[truncated].../{parent_str}/{}",
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
