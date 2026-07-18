use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Borders},
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

pub fn render(frame: &mut Frame<'_>) {
    let layout = two_pane_layout(frame.area());
    frame.render_widget(
        Block::default().borders(Borders::ALL).title("Files"),
        layout.file_tree,
    );
    frame.render_widget(
        Block::default().borders(Borders::ALL).title("Preview"),
        layout.preview,
    );
}

#[cfg(test)]
mod tests {
    use super::two_pane_layout;
    use ratatui::layout::Rect;

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
}
