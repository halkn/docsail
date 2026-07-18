use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use unicode_width::UnicodeWidthChar;

use crate::{
    app::Focus,
    markdown::{Block as MarkdownBlock, Document, HeadingLevel, Inline},
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

pub fn render(
    frame: &mut Frame<'_>,
    tree: &FileTree,
    selected_file_index: usize,
    focus: Focus,
    document: &Document,
) {
    let layout = two_pane_layout(frame.area());
    render_file_tree(
        frame,
        layout.file_tree,
        tree,
        selected_file_index,
        focus == Focus::FileTree,
    );
    render_preview(frame, layout.preview, focus == Focus::Preview, document);
}

fn render_preview(frame: &mut Frame<'_>, area: Rect, is_focused: bool, document: &Document) {
    let width = usize::from(area.width.saturating_sub(2));
    let lines = document
        .blocks()
        .iter()
        .filter_map(|block| match block {
            MarkdownBlock::Heading { level, content } => Some((
                format!("{} {}", heading_marker(*level), inline_text(content)),
                Style::default().add_modifier(Modifier::BOLD),
            )),
            MarkdownBlock::Paragraph(content) => Some((inline_text(content), Style::default())),
            MarkdownBlock::Table { header, rows } => {
                Some((table_text(header, rows), Style::default()))
            }
            _ => None,
        })
        .flat_map(|(text, style)| {
            wrap_unicode(&text, width)
                .into_iter()
                .map(move |line| Line::from(Span::styled(line, style)))
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(lines).block(pane_block("Preview", is_focused)),
        area,
    );
}

fn wrap_unicode(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut line_width = 0;
    for character in text.chars() {
        if character == '\n' {
            lines.push(std::mem::take(&mut line));
            line_width = 0;
            continue;
        }
        let character_width = character.width().unwrap_or(0);
        if line_width > 0 && line_width + character_width > width {
            lines.push(std::mem::take(&mut line));
            line_width = 0;
        }
        line.push(character);
        line_width += character_width;
    }
    lines.push(line);
    lines
}

fn table_text(header: &[Vec<Inline>], rows: &[Vec<Vec<Inline>>]) -> String {
    std::iter::once(header)
        .chain(rows.iter().map(Vec::as_slice))
        .map(|row| {
            format!(
                "| {} |",
                row.iter()
                    .map(|cell| inline_text(cell))
                    .collect::<Vec<_>>()
                    .join(" | ")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn heading_marker(level: HeadingLevel) -> &'static str {
    match level {
        HeadingLevel::One => "#",
        HeadingLevel::Two => "##",
        HeadingLevel::Three => "###",
        HeadingLevel::Four => "####",
        HeadingLevel::Five => "#####",
        HeadingLevel::Six => "######",
    }
}

fn inline_text(inlines: &[Inline]) -> String {
    inlines.iter().fold(String::new(), |mut text, inline| {
        match inline {
            Inline::Text(value) | Inline::Code(value) | Inline::Autolink(value) => {
                text.push_str(value)
            }
            Inline::SoftBreak | Inline::HardBreak => text.push('\n'),
            _ => {}
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
    use super::{file_tree_rows, two_pane_layout, wrap_unicode};
    use crate::workspace::FileTree;
    use ratatui::layout::Rect;
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
    fn wraps_japanese_text_at_terminal_display_width() {
        assert_eq!(wrap_unicode("日本語abc", 6), ["日本語", "abc"]);
    }
}
