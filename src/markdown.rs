use pulldown_cmark::{
    CodeBlockKind, Event, HeadingLevel as ParserHeadingLevel, Parser, Tag, TagEnd,
};

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
    let mut code_block = None;

    for event in Parser::new(source) {
        match event {
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
            Event::Start(Tag::CodeBlock(kind)) => {
                let language = match kind {
                    CodeBlockKind::Fenced(language) if !language.is_empty() => {
                        Some(language.into_string())
                    }
                    _ => None,
                };
                code_block = Some((language, String::new()));
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some((language, content)) = code_block.take() {
                    blocks.push(Block::CodeBlock { language, content });
                }
            }
            Event::Text(text) => {
                if let Some((_, content)) = &mut code_block {
                    content.push_str(&text);
                } else {
                    push_inline(&mut active_block, Inline::Text(text.into_string()));
                }
            }
            Event::Code(code) => push_inline(&mut active_block, Inline::Code(code.into_string())),
            Event::SoftBreak => push_inline(&mut active_block, Inline::SoftBreak),
            Event::HardBreak => push_inline(&mut active_block, Inline::HardBreak),
            _ => {}
        }
    }

    Document::new(blocks)
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
    fn parses_inline_and_fenced_code() {
        let document = parse("Use `cargo test`.\n\n```rust\nfn main() {}\n```");
        assert!(
            matches!(&document.blocks()[0], Block::Paragraph(content) if matches!(content[1], Inline::Code(_)))
        );
        assert!(
            matches!(&document.blocks()[1], Block::CodeBlock { language: Some(language), content } if language == "rust" && content.contains("fn main"))
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
