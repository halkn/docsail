use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use unicode_width::UnicodeWidthStr;

use crate::{
    app::{Focus, Overlay},
    markdown::{Block as MarkdownBlock, Document, HeadingLevel, Inline, TableAlignment},
    workspace::{FileTree, FileTreeNode},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TwoPaneLayout {
    pub file_tree: Rect,
    pub preview: Rect,
}

pub fn two_pane_layout(area: Rect) -> TwoPaneLayout {
    let [file_tree, preview] =
        Layout::horizontal([Constraint::Percentage(35), Constraint::Percentage(65)]).areas(area);

    TwoPaneLayout { file_tree, preview }
}

pub struct RenderState<'a> {
    pub selected_file_index: usize,
    pub focus: Focus,
    pub preview_scroll: usize,
    pub document: &'a Document,
    pub overlay: Option<Overlay>,
    pub query: &'a str,
    pub query_cursor: usize,
    pub overlay_results: &'a [String],
    pub selected_result: usize,
}

pub fn render(frame: &mut Frame<'_>, tree: &FileTree, state: RenderState<'_>) {
    let layout = two_pane_layout(frame.area());
    render_file_tree(
        frame,
        layout.file_tree,
        tree,
        state.selected_file_index,
        state.focus == Focus::FileTree,
    );
    render_preview(
        frame,
        layout.preview,
        state.preview_scroll,
        state.focus == Focus::Preview,
        state.document,
    );
    if let Some(overlay) = state.overlay {
        render_overlay(
            frame,
            overlay,
            state.query,
            state.query_cursor,
            state.overlay_results,
            state.selected_result,
        );
    }
}

pub fn preview_scroll_for_block(document: &Document, block_index: usize) -> usize {
    document
        .blocks()
        .iter()
        .take(block_index)
        .map(|block| block_lines(block).len() + 1)
        .sum()
}

fn render_overlay(
    frame: &mut Frame<'_>,
    overlay: Overlay,
    query: &str,
    query_cursor: usize,
    results: &[String],
    selected: usize,
) {
    let title = match overlay {
        Overlay::Toc => "Table of Contents",
        Overlay::FileSearch => "Find file",
        Overlay::PageSearch => "Find in page",
    };
    let area = centered_rect(70, 60, frame.area());
    frame.render_widget(ratatui::widgets::Clear, area);
    let mut lines = vec![Line::from(format!("> {query}"))];
    lines.extend(results.iter().enumerate().map(|(index, result)| {
        Line::from(Span::styled(
            result.clone(),
            if index == selected {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            },
        ))
    }));
    frame.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(title)),
        area,
    );
    if !matches!(overlay, Overlay::Toc) {
        let (x, y) = overlay_cursor_position(area, &query[..query_cursor]);
        frame.set_cursor_position((x, y));
    }
}

fn overlay_cursor_position(area: Rect, query: &str) -> (u16, u16) {
    let input_start = area.x.saturating_add(3);
    let input_end = area.right().saturating_sub(1);
    let query_width = u16::try_from(query.width()).unwrap_or(u16::MAX);
    (
        input_start.saturating_add(query_width).min(input_end),
        area.y.saturating_add(1),
    )
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);
    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}

fn render_preview(
    frame: &mut Frame<'_>,
    area: Rect,
    preview_scroll: usize,
    is_focused: bool,
    document: &Document,
) {
    let lines = preview_lines(document);
    frame.render_widget(
        Paragraph::new(lines)
            .scroll((preview_scroll_offset(preview_scroll), 0))
            .wrap(Wrap { trim: false })
            .block(pane_block("Preview", is_focused)),
        area,
    );
}

fn preview_lines(document: &Document) -> Vec<Line<'static>> {
    document
        .blocks()
        .iter()
        .enumerate()
        .flat_map(|(index, block)| {
            let mut lines = Vec::new();
            if index > 0 {
                lines.push(Line::default());
            }
            lines.extend(block_lines(block));
            lines
        })
        .collect()
}

fn block_lines(block: &MarkdownBlock) -> Vec<Line<'static>> {
    match block {
        MarkdownBlock::Heading { level, content } => heading_lines(*level, content),
        MarkdownBlock::Paragraph(content) => inline_lines(content, Style::default()),
        MarkdownBlock::CodeBlock { language, content } => {
            let mut lines = Vec::new();
            if let Some(language) = language {
                lines.push(Line::from(Span::styled(
                    language.clone(),
                    Style::default().fg(Color::DarkGray),
                )));
            }
            lines.extend(content.lines().map(|line| {
                Line::from(Span::styled(
                    line.to_owned(),
                    Style::default().fg(Color::Yellow),
                ))
            }));
            lines
        }
        MarkdownBlock::List { start, items } => items
            .iter()
            .enumerate()
            .flat_map(|(index, item)| {
                let prefix = match (start, item.task) {
                    (_, Some(true)) => "[x] ".to_owned(),
                    (_, Some(false)) => "[ ] ".to_owned(),
                    (Some(start), None) => format!("{}. ", start + index as u64),
                    (None, None) => "• ".to_owned(),
                };
                let continuation = " ".repeat(prefix.width());
                let lines = item.blocks.iter().flat_map(block_lines).collect::<Vec<_>>();
                lines
                    .into_iter()
                    .enumerate()
                    .map(|(line_index, mut line)| {
                        line.spans.insert(
                            0,
                            Span::raw(if line_index == 0 {
                                prefix.clone()
                            } else {
                                continuation.clone()
                            }),
                        );
                        line
                    })
                    .collect::<Vec<_>>()
            })
            .collect(),
        MarkdownBlock::BlockQuote(blocks) => blocks
            .iter()
            .flat_map(block_lines)
            .map(|mut line| {
                line.spans
                    .insert(0, Span::styled("│ ", Style::default().fg(Color::DarkGray)));
                line
            })
            .collect(),
        MarkdownBlock::Table {
            header,
            rows,
            alignments,
        } => table_lines(header, rows, alignments),
        MarkdownBlock::ThematicBreak => vec![Line::from(Span::styled(
            "─".repeat(20),
            Style::default().fg(Color::DarkGray),
        ))],
        MarkdownBlock::Html(html) => vec![Line::from(Span::styled(
            html.clone(),
            Style::default().fg(Color::DarkGray),
        ))],
    }
}

fn heading_lines(level: HeadingLevel, content: &[Inline]) -> Vec<Line<'static>> {
    let color = match level {
        HeadingLevel::One => Color::Magenta,
        HeadingLevel::Two => Color::Cyan,
        HeadingLevel::Three => Color::Green,
        HeadingLevel::Four => Color::Yellow,
        HeadingLevel::Five => Color::Blue,
        HeadingLevel::Six => Color::Gray,
    };
    let title_style = Style::default().fg(color).add_modifier(Modifier::BOLD);
    let mut title = inline_lines(content, title_style);

    match level {
        HeadingLevel::One => {
            let width = inline_plain_text(content).width().max(20);
            let mut lines = vec![Line::from(Span::styled(
                "═".repeat(width),
                Style::default().fg(color),
            ))];
            lines.append(&mut title);
            lines.push(Line::from(Span::styled(
                "═".repeat(width),
                Style::default().fg(color),
            )));
            lines
        }
        HeadingLevel::Two => {
            let width = inline_plain_text(content).width().max(16);
            title.push(Line::from(Span::styled(
                "─".repeat(width),
                Style::default().fg(color),
            )));
            title
        }
        HeadingLevel::Three => title,
        HeadingLevel::Four | HeadingLevel::Five | HeadingLevel::Six => {
            title[0]
                .spans
                .insert(0, Span::styled("▸ ", Style::default().fg(color)));
            title
        }
    }
}

fn inline_lines(inlines: &[Inline], style: Style) -> Vec<Line<'static>> {
    let mut lines = vec![Vec::new()];
    for inline in inlines {
        if matches!(inline, Inline::HardBreak) {
            lines.push(Vec::new());
        } else {
            lines
                .last_mut()
                .expect("lines is never empty")
                .extend(inline_spans(inline, style));
        }
    }
    lines.into_iter().map(Line::from).collect()
}

fn inline_spans(inline: &Inline, style: Style) -> Vec<Span<'static>> {
    match inline {
        Inline::Text(value) => vec![Span::styled(value.clone(), style)],
        Inline::SoftBreak => vec![Span::styled(" ", style)],
        Inline::HardBreak => Vec::new(),
        Inline::Html(value) => vec![Span::styled(
            value.clone(),
            Style::default().fg(Color::DarkGray),
        )],
        Inline::Code(value) => vec![Span::styled(
            value.clone(),
            style.fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )],
        Inline::Emphasis(content) => {
            inline_spans_for(content, style.add_modifier(Modifier::ITALIC))
        }
        Inline::Strong(content) => inline_spans_for(content, style.add_modifier(Modifier::BOLD)),
        Inline::Strikethrough(content) => {
            inline_spans_for(content, style.add_modifier(Modifier::CROSSED_OUT))
        }
        Inline::Link { content, .. } => inline_spans_for(content, link_style(style)),
        Inline::Autolink(destination) => vec![Span::styled(destination.clone(), link_style(style))],
        Inline::Image { alt, .. } => {
            let mut spans = vec![Span::styled(
                "[image: ",
                Style::default().fg(Color::DarkGray),
            )];
            spans.extend(inline_spans_for(alt, style));
            spans.push(Span::styled("]", Style::default().fg(Color::DarkGray)));
            spans
        }
    }
}

fn inline_spans_for(inlines: &[Inline], style: Style) -> Vec<Span<'static>> {
    inlines
        .iter()
        .flat_map(|inline| inline_spans(inline, style))
        .collect()
}

fn link_style(style: Style) -> Style {
    style.fg(Color::Blue).add_modifier(Modifier::UNDERLINED)
}

fn preview_scroll_offset(scroll: usize) -> u16 {
    u16::try_from(scroll).unwrap_or(u16::MAX)
}

fn table_lines(
    header: &[Vec<Inline>],
    rows: &[Vec<Vec<Inline>>],
    alignments: &[TableAlignment],
) -> Vec<Line<'static>> {
    let column_count = std::iter::once(header.len())
        .chain(rows.iter().map(Vec::len))
        .max()
        .unwrap_or(0);
    if column_count == 0 {
        return Vec::new();
    }

    let all_rows = std::iter::once(header).chain(rows.iter().map(Vec::as_slice));
    let widths = (0..column_count)
        .map(|column| {
            all_rows
                .clone()
                .filter_map(|row| row.get(column))
                .map(|cell| inline_plain_text(cell).width())
                .max()
                .unwrap_or(0)
        })
        .collect::<Vec<_>>();

    let mut lines = vec![table_border('┌', '┬', '┐', &widths)];
    lines.push(table_row(header, &widths, alignments, true));
    lines.push(table_border('├', '┼', '┤', &widths));
    lines.extend(
        rows.iter()
            .map(|row| table_row(row, &widths, alignments, false)),
    );
    lines.push(table_border('└', '┴', '┘', &widths));
    lines
}

fn table_border(left: char, divider: char, right: char, widths: &[usize]) -> Line<'static> {
    Line::from(Span::styled(
        format!(
            "{left}{}{right}",
            widths
                .iter()
                .map(|width| "─".repeat(width + 2))
                .collect::<Vec<_>>()
                .join(&divider.to_string())
        ),
        Style::default().fg(Color::DarkGray),
    ))
}

fn table_row(
    row: &[Vec<Inline>],
    widths: &[usize],
    alignments: &[TableAlignment],
    is_header: bool,
) -> Line<'static> {
    let cell_style = if is_header {
        Style::default().add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let mut spans = vec![Span::styled("│", Style::default().fg(Color::DarkGray))];
    for (column, width) in widths.iter().enumerate() {
        let text = row
            .get(column)
            .map_or_else(String::new, |cell| inline_plain_text(cell));
        let padding = width.saturating_sub(text.width());
        let (left, right) = match alignments
            .get(column)
            .copied()
            .unwrap_or(TableAlignment::None)
        {
            TableAlignment::Right => (padding, 0),
            TableAlignment::Center => (padding / 2, padding - padding / 2),
            _ => (0, padding),
        };
        spans.push(Span::raw(" ".repeat(left + 1)));
        spans.push(Span::styled(text, cell_style));
        spans.push(Span::raw(" ".repeat(right + 1)));
        spans.push(Span::styled("│", Style::default().fg(Color::DarkGray)));
    }
    Line::from(spans)
}

fn inline_plain_text(inlines: &[Inline]) -> String {
    inlines.iter().fold(String::new(), |mut text, inline| {
        match inline {
            Inline::Text(value) => text.push_str(value),
            Inline::Autolink(value) | Inline::Code(value) => text.push_str(value),
            Inline::Link { content, .. }
            | Inline::Emphasis(content)
            | Inline::Strong(content)
            | Inline::Strikethrough(content) => text.push_str(&inline_plain_text(content)),
            Inline::Image { alt, .. } => {
                text.push_str("[image: ");
                text.push_str(&inline_plain_text(alt));
                text.push(']');
            }
            Inline::SoftBreak => text.push(' '),
            Inline::HardBreak => text.push('\n'),
            Inline::Html(value) => text.push_str(value),
        }
        text
    })
}

fn render_file_tree(
    frame: &mut Frame<'_>,
    area: Rect,
    tree: &FileTree,
    selected_file_index: usize,
    is_focused: bool,
) {
    let rows = file_tree_rows(tree, selected_file_index);
    let lines = rows
        .into_iter()
        .map(FileTreeRow::into_line)
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(lines).block(pane_block("Files", is_focused)),
        area,
    );
}

fn pane_block(title: &str, is_focused: bool) -> Block<'_> {
    let style = if is_focused {
        Style::default().fg(ratatui::style::Color::Cyan)
    } else {
        Style::default()
    };
    Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(style)
}

#[derive(Debug, PartialEq, Eq)]
struct FileTreeRow {
    label: String,
    is_selected: bool,
}

impl FileTreeRow {
    fn into_line(self) -> Line<'static> {
        let style = if self.is_selected {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
        };

        Line::from(Span::styled(self.label, style))
    }
}

fn file_tree_rows(tree: &FileTree, selected_file_index: usize) -> Vec<FileTreeRow> {
    let mut rows = Vec::new();
    let mut file_index = 0;
    append_tree_rows(
        tree.root().children(),
        0,
        selected_file_index,
        &mut file_index,
        &mut rows,
    );
    rows
}

fn append_tree_rows(
    nodes: &[FileTreeNode],
    depth: usize,
    selected_file_index: usize,
    file_index: &mut usize,
    rows: &mut Vec<FileTreeRow>,
) {
    for node in nodes {
        let indent = "  ".repeat(depth);
        match node {
            FileTreeNode::Directory { name, children, .. } => {
                rows.push(FileTreeRow {
                    label: format!("{indent}▾ {}/", name.to_string_lossy()),
                    is_selected: false,
                });
                append_tree_rows(children, depth + 1, selected_file_index, file_index, rows);
            }
            FileTreeNode::Page { name, children, .. } => {
                rows.push(FileTreeRow {
                    label: format!("{indent}▾ {}", name.to_string_lossy()),
                    is_selected: *file_index == selected_file_index,
                });
                *file_index += 1;
                append_tree_rows(children, depth + 1, selected_file_index, file_index, rows);
            }
            FileTreeNode::File { name, .. } => {
                rows.push(FileTreeRow {
                    label: format!("{indent}  {}", name.to_string_lossy()),
                    is_selected: *file_index == selected_file_index,
                });
                *file_index += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        file_tree_rows, overlay_cursor_position, preview_lines, preview_scroll_for_block,
        preview_scroll_offset, two_pane_layout,
    };
    use crate::markdown::parse;
    use crate::workspace::FileTree;
    use ratatui::{layout::Rect, style::Color};
    use std::path::PathBuf;

    #[test]
    fn splits_the_available_width_between_file_tree_and_preview() {
        let layout = two_pane_layout(Rect::new(4, 2, 100, 30));

        assert_eq!(layout.file_tree, Rect::new(4, 2, 35, 30));
        assert_eq!(layout.preview, Rect::new(39, 2, 65, 30));
    }

    #[test]
    fn keeps_panes_within_a_narrow_area_without_overlap() {
        let area = Rect::new(0, 0, 1, 10);
        let layout = two_pane_layout(area);

        assert_eq!(layout.file_tree.x, area.x);
        assert_eq!(layout.preview.right(), area.right());
        assert_eq!(layout.file_tree.right(), layout.preview.x);
        assert_eq!(layout.file_tree.height, area.height);
        assert_eq!(layout.preview.height, area.height);
    }

    #[test]
    fn renders_directories_and_marks_the_selected_file() {
        let root = PathBuf::from("workspace");
        let tree = FileTree::from_files(
            &root,
            vec![root.join("guide/setup.md"), root.join("README.md")],
        )
        .unwrap();

        let rows = file_tree_rows(&tree, 1);

        assert_eq!(rows[0].label, "  README.md");
        assert!(!rows[0].is_selected);
        assert_eq!(rows[1].label, "▾ guide/");
        assert_eq!(rows[2].label, "    setup.md");
        assert!(rows[2].is_selected);
    }

    #[test]
    fn clamps_preview_scroll_to_the_terminal_coordinate_range() {
        assert_eq!(preview_scroll_offset(12), 12);
        assert_eq!(preview_scroll_offset(usize::MAX), u16::MAX);
    }

    #[test]
    fn offsets_the_preview_for_the_blank_line_before_a_later_block() {
        let document = parse("first\n\nsecond");

        assert_eq!(preview_scroll_for_block(&document, 0), 0);
        assert_eq!(preview_scroll_for_block(&document, 1), 2);
    }

    #[test]
    fn positions_the_input_cursor_after_wide_characters() {
        let area = Rect::new(10, 4, 20, 8);

        assert_eq!(overlay_cursor_position(area, "日本"), (17, 5));
        assert_eq!(overlay_cursor_position(area, &"x".repeat(30)), (29, 5));
    }

    #[test]
    fn renders_markdown_as_a_preview_without_source_markers() {
        let document = parse(
            "# Title\n\nfirst line\nsecond line\n\n| Name | Value |\n| --- | --- |\n| DocSail | TUI |",
        );
        let lines = preview_lines(&document)
            .iter()
            .map(line_text)
            .collect::<Vec<_>>();

        assert_eq!(lines[0], "════════════════════");
        assert_eq!(lines[1], "Title");
        assert_eq!(lines[2], "════════════════════");
        assert_eq!(lines[4], "first line second line");
        assert_eq!(lines[6], "┌─────────┬───────┐");
        assert_eq!(lines[7], "│ Name    │ Value │");
        assert!(lines.iter().all(|line| !line.starts_with("# ")));
        assert!(lines.iter().all(|line| !line.starts_with("| ")));
    }

    #[test]
    fn renders_unordered_ordered_and_task_lists() {
        let document = parse("- first\n- second\n\n1. one\n2. two\n\n- [x] done\n- [ ] remaining");
        let lines = preview_lines(&document)
            .iter()
            .map(line_text)
            .collect::<Vec<_>>();

        assert_eq!(
            lines,
            [
                "• first",
                "• second",
                "",
                "1. one",
                "2. two",
                "",
                "[x] done",
                "[ ] remaining",
            ]
        );
    }

    #[test]
    fn differentiates_heading_levels_without_markdown_markers() {
        let document = parse("# One\n\n## Two\n\n### Three\n\n#### Four");
        let rendered = preview_lines(&document);
        let lines = rendered.iter().map(line_text).collect::<Vec<_>>();

        assert_eq!(lines[1], "One");
        assert_eq!(lines[4], "Two");
        assert_eq!(lines[5], "────────────────");
        assert_eq!(lines[7], "Three");
        assert_eq!(lines[9], "▸ Four");
        assert!(lines.iter().all(|line| !line.starts_with("#")));
        assert_eq!(rendered[1].spans[0].style.fg, Some(Color::Magenta));
        assert_eq!(rendered[4].spans[0].style.fg, Some(Color::Cyan));
        assert_eq!(rendered[7].spans[0].style.fg, Some(Color::Green));
        assert_eq!(rendered[9].spans[0].style.fg, Some(Color::Yellow));
    }

    fn line_text(line: &ratatui::text::Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }
}
