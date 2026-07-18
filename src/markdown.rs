#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Document {
    blocks: Vec<Block>,
}

impl Document {
    pub fn new(blocks: Vec<Block>) -> Self {
        Self { blocks }
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    Heading {
        level: HeadingLevel,
        content: Vec<Inline>,
    },
    Paragraph(Vec<Inline>),
    List {
        ordered: bool,
        items: Vec<ListItem>,
    },
    BlockQuote(Vec<Block>),
    CodeBlock {
        language: Option<String>,
        content: String,
    },
    Table {
        header: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
    ThematicBreak,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadingLevel {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListItem {
    pub task: Option<bool>,
    pub blocks: Vec<Block>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inline {
    Text(String),
    Emphasis(Vec<Inline>),
    Strong(Vec<Inline>),
    Strikethrough(Vec<Inline>),
    Code(String),
    Link {
        content: Vec<Inline>,
        destination: String,
    },
    Image {
        alt: Vec<Inline>,
        destination: String,
    },
    Autolink(String),
    SoftBreak,
    HardBreak,
}

#[cfg(test)]
mod tests {
    use super::{Block, Document, HeadingLevel, Inline, ListItem};

    #[test]
    fn preserves_blocks_and_nested_gfm_content_without_rendering_types() {
        let document = Document::new(vec![
            Block::Heading {
                level: HeadingLevel::One,
                content: vec![Inline::Text("DocSail".to_owned())],
            },
            Block::List {
                ordered: false,
                items: vec![ListItem {
                    task: Some(true),
                    blocks: vec![Block::Paragraph(vec![Inline::Strikethrough(vec![
                        Inline::Text("done".to_owned()),
                    ])])],
                }],
            },
        ]);

        assert_eq!(document.blocks().len(), 2);
        assert!(matches!(document.blocks()[0], Block::Heading { .. }));
        assert!(matches!(document.blocks()[1], Block::List { .. }));
    }
}
