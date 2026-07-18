use pulldown_cmark::{Event, HeadingLevel as ParserHeadingLevel, Parser, Tag, TagEnd};

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

pub fn parse(source: &str) -> Document {
    let mut blocks = Vec::new();
    let mut active_block = None;
    let mut inline_spans = Vec::new();

    for event in Parser::new(source) {
        match event {
            Event::Start(Tag::Link { dest_url, .. }) => {
                if let Some(content) = active_content(&mut active_block) {
                    inline_spans.push(InlineSpan::Link {
                        start: content.len(),
                        destination: dest_url.into_string(),
                    });
                }
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                if let Some(content) = active_content(&mut active_block) {
                    inline_spans.push(InlineSpan::Image {
                        start: content.len(),
                        destination: dest_url.into_string(),
                    });
                }
            }
            Event::End(TagEnd::Link) | Event::End(TagEnd::Image) => {
                if let Some(span) = inline_spans.pop()
                    && let Some(content) = active_content(&mut active_block)
                {
                    span.wrap(content);
                }
            }
            Event::Start(Tag::Heading { level, .. }) => {
                active_block = Some(ActiveBlock::Heading {
                    level: heading_level(level),
                    content: Vec::new(),
                });
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(ActiveBlock::Heading { level, content }) = active_block.take() {
                    blocks.push(Block::Heading { level, content });
                }
            }
            Event::Start(Tag::Paragraph) => {
                active_block = Some(ActiveBlock::Paragraph(Vec::new()));
            }
            Event::End(TagEnd::Paragraph) => {
                if let Some(ActiveBlock::Paragraph(content)) = active_block.take() {
                    blocks.push(Block::Paragraph(content));
                }
            }
            Event::Text(text) => push_inline(&mut active_block, Inline::Text(text.into_string())),
            Event::Code(code) => push_inline(&mut active_block, Inline::Code(code.into_string())),
            Event::SoftBreak => push_inline(&mut active_block, Inline::SoftBreak),
            Event::HardBreak => push_inline(&mut active_block, Inline::HardBreak),
            _ => {}
        }
    }

    Document::new(blocks)
}

enum InlineSpan {
    Link { start: usize, destination: String },
    Image { start: usize, destination: String },
}
impl InlineSpan {
    fn wrap(self, content: &mut Vec<Inline>) {
        let (start, destination, image) = match self {
            Self::Link { start, destination } => (start, destination, false),
            Self::Image { start, destination } => (start, destination, true),
        };
        let nested = content.split_off(start);
        content.push(if image {
            Inline::Image {
                alt: nested,
                destination,
            }
        } else {
            Inline::Link {
                content: nested,
                destination,
            }
        });
    }
}

fn active_content(active_block: &mut Option<ActiveBlock>) -> Option<&mut Vec<Inline>> {
    match active_block {
        Some(ActiveBlock::Heading { content, .. }) | Some(ActiveBlock::Paragraph(content)) => {
            Some(content)
        }
        None => None,
    }
}

enum ActiveBlock {
    Heading {
        level: HeadingLevel,
        content: Vec<Inline>,
    },
    Paragraph(Vec<Inline>),
}

fn push_inline(active_block: &mut Option<ActiveBlock>, inline: Inline) {
    match active_block {
        Some(ActiveBlock::Heading { content, .. }) | Some(ActiveBlock::Paragraph(content)) => {
            content.push(inline)
        }
        None => {}
    }
}

fn heading_level(level: ParserHeadingLevel) -> HeadingLevel {
    match level {
        ParserHeadingLevel::H1 => HeadingLevel::One,
        ParserHeadingLevel::H2 => HeadingLevel::Two,
        ParserHeadingLevel::H3 => HeadingLevel::Three,
        ParserHeadingLevel::H4 => HeadingLevel::Four,
        ParserHeadingLevel::H5 => HeadingLevel::Five,
        ParserHeadingLevel::H6 => HeadingLevel::Six,
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
    use super::{Block, Document, HeadingLevel, Inline, ListItem, parse};

    #[test]
    fn parses_headings_and_paragraphs() {
        let document = parse("# DocSail\n\nA terminal Markdown viewer.");

        assert_eq!(
            document.blocks(),
            [
                Block::Heading {
                    level: HeadingLevel::One,
                    content: vec![Inline::Text("DocSail".to_owned())],
                },
                Block::Paragraph(vec![Inline::Text("A terminal Markdown viewer.".to_owned())]),
            ]
        );
    }

    #[test]
    fn parses_links_and_images() {
        let document = parse("[DocSail](https://example.invalid) ![Logo](logo.png)");
        assert!(
            matches!(&document.blocks()[0], Block::Paragraph(content) if matches!(content[0], Inline::Link { .. }) && matches!(content[2], Inline::Image { .. }))
        );
    }

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
