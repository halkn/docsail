use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
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
    document: &Document,
) {
    let layout = two_pane_layout(frame.area());
    render_file_tree(frame, layout.file_tree, tree, selected_file_index);
    render_preview(frame, layout.preview, document);
}

fn render_preview(frame: &mut Frame<'_>, area: Rect, document: &Document) {
    let lines = document
        .blocks()
        .iter()
        .filter_map(|block| match block {
            MarkdownBlock::Heading { level, content } => Some(Line::from(Span::styled(
                format!("{} {}", heading_marker(*level), inline_text(content)),
                Style::default().add_modifier(Modifier::BOLD),
            ))),
            MarkdownBlock::Paragraph(content) => Some(Line::from(inline_text(content))),
            MarkdownBlock::CodeBlock { language, content } => Some(Line::from(format!(
                "```{}\n{}\n```",
                language.as_deref().unwrap_or(""),
                content
            ))),
            _ => None,
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Preview")),
        area,
    );
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
            Inline::Text(value) | Inline::Autolink(value) => text.push_str(value),
            Inline::Code(value) => {
                text.push('`');
                text.push_str(value);
                text.push('`');
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
) {
    let rows = file_tree_rows(tree, selected_file_index);
    let lines = rows
        .into_iter()
        .map(FileTreeRow::into_line)
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Files")),
        area,
    );
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
    use super::{file_tree_rows, two_pane_layout};
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
}
