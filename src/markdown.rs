use pulldown_cmark::{Event, HeadingLevel as ParserHeadingLevel, Options, Parser, Tag, TagEnd};

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
    let mut list = None;
    let mut quote_blocks = None;

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TASKLISTS);
    for event in Parser::new_ext(source, options) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                active_block = Some(ActiveBlock::Heading {
                    level: heading_level(level),
                    content: Vec::new(),
                });
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(ActiveBlock::Heading { level, content }) = active_block.take() {
                    push_block(
                        &mut blocks,
                        &mut list,
                        &mut quote_blocks,
                        Block::Heading { level, content },
                    );
                }
            }
            Event::Start(Tag::Paragraph) => {
                active_block = Some(ActiveBlock::Paragraph(Vec::new()));
            }
            Event::End(TagEnd::Paragraph) => {
                if let Some(ActiveBlock::Paragraph(content)) = active_block.take() {
                    push_block(
                        &mut blocks,
                        &mut list,
                        &mut quote_blocks,
                        Block::Paragraph(content),
                    );
                }
            }
            Event::Start(Tag::List(start)) => list = Some(ListContext::new(start.is_some())),
            Event::Start(Tag::Item) => {
                if let Some(list) = &mut list {
                    list.current = Some(ListItem {
                        task: None,
                        blocks: Vec::new(),
                    });
                }
            }
            Event::TaskListMarker(checked) => {
                if let Some(Some(item)) = list.as_mut().map(|list| list.current.as_mut()) {
                    item.task = Some(checked);
                }
            }
            Event::End(TagEnd::Item) => {
                if let Some(list) = &mut list
                    && let Some(item) = list.current.take()
                {
                    list.items.push(item);
                }
            }
            Event::End(TagEnd::List(_)) => {
                if let Some(list) = list.take() {
                    push_block(
                        &mut blocks,
                        &mut None,
                        &mut quote_blocks,
                        Block::List {
                            ordered: list.ordered,
                            items: list.items,
                        },
                    );
                }
            }
            Event::Start(Tag::BlockQuote(_)) => quote_blocks = Some(Vec::new()),
            Event::End(TagEnd::BlockQuote(_)) => {
                if let Some(quoted) = quote_blocks.take() {
                    blocks.push(Block::BlockQuote(quoted));
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

struct ListContext {
    ordered: bool,
    items: Vec<ListItem>,
    current: Option<ListItem>,
}
impl ListContext {
    fn new(ordered: bool) -> Self {
        Self {
            ordered,
            items: Vec::new(),
            current: None,
        }
    }
}

fn push_block(
    blocks: &mut Vec<Block>,
    list: &mut Option<ListContext>,
    quote: &mut Option<Vec<Block>>,
    block: Block,
) {
    if let Some(Some(item)) = list.as_mut().map(|list| list.current.as_mut()) {
        item.blocks.push(block);
    } else if let Some(quoted) = quote {
        quoted.push(block);
    } else {
        blocks.push(block);
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
    fn parses_lists_tasks_and_blockquotes() {
        let document = parse("- [x] done\n- next\n\n> quoted");
        assert!(
            matches!(&document.blocks()[0], Block::List { items, .. } if items[0].task == Some(true))
        );
        assert!(matches!(&document.blocks()[1], Block::BlockQuote(_)));
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
