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
    let mut table = None;
    let mut inline_spans = Vec::new();

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_TABLES);
    for event in Parser::new_ext(source, options) {
        match event {
            Event::Start(Tag::Link { dest_url, .. }) => {
                if let Some(content) = active_content(&mut active_block, &mut table) {
                    inline_spans.push(InlineSpan::Link {
                        start: content.len(),
                        destination: dest_url.into_string(),
                    });
                }
            }
            Event::Start(Tag::Image { dest_url, .. }) => {
                if let Some(content) = active_content(&mut active_block, &mut table) {
                    inline_spans.push(InlineSpan::Image {
                        start: content.len(),
                        destination: dest_url.into_string(),
                    });
                }
            }
            Event::End(TagEnd::Link) | Event::End(TagEnd::Image) => {
                if let Some(span) = inline_spans.pop()
                    && let Some(content) = active_content(&mut active_block, &mut table)
                {
                    span.wrap(content);
                }
            }
            Event::Start(Tag::Table(_)) => table = Some(TableContext::default()),
            Event::Start(Tag::TableHead) => {
                if let Some(table) = &mut table {
                    table.in_header = true;
                    table.current_row = Some(Vec::new());
                }
            }
            Event::End(TagEnd::TableHead) => {
                if let Some(table) = &mut table {
                    if let Some(row) = table.current_row.take() {
                        table.header = row;
                    }
                    table.in_header = false;
                }
            }
            Event::Start(Tag::TableRow) => {
                if let Some(table) = &mut table {
                    table.current_row = Some(Vec::new());
                }
            }
            Event::End(TagEnd::TableRow) => {
                if let Some(table) = &mut table
                    && let Some(row) = table.current_row.take()
                {
                    if table.in_header {
                        table.header = row;
                    } else {
                        table.rows.push(row);
                    }
                }
            }
            Event::Start(Tag::TableCell) => {
                if let Some(table) = &mut table {
                    table.current_cell = Some(Vec::new());
                }
            }
            Event::End(TagEnd::TableCell) => {
                if let Some(table) = &mut table
                    && let Some(cell) = table.current_cell.take()
                    && let Some(row) = &mut table.current_row
                {
                    row.push(cell);
                }
            }
            Event::End(TagEnd::Table) => {
                if let Some(table) = table.take() {
                    blocks.push(Block::Table {
                        header: table.header,
                        rows: table.rows,
                    });
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
            Event::Text(text) => {
                if let Some(Some(cell)) = table.as_mut().map(|table| table.current_cell.as_mut()) {
                    cell.push(Inline::Text(text.into_string()));
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

fn active_content<'a>(
    active_block: &'a mut Option<ActiveBlock>,
    table: &'a mut Option<TableContext>,
) -> Option<&'a mut Vec<Inline>> {
    if let Some(table) = table.as_mut()
        && let Some(cell) = table.current_cell.as_mut()
    {
        return Some(cell);
    }
    match active_block {
        Some(ActiveBlock::Heading { content, .. }) | Some(ActiveBlock::Paragraph(content)) => {
            Some(content)
        }
        None => None,
    }
}

#[derive(Default)]
struct TableContext {
    header: Vec<Vec<Inline>>,
    rows: Vec<Vec<Vec<Inline>>>,
    current_row: Option<Vec<Vec<Inline>>>,
    current_cell: Option<Vec<Inline>>,
    in_header: bool,
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
    fn parses_links_and_images() {
        let document = parse("[DocSail](https://example.invalid) ![Logo](logo.png)");
        assert!(
            matches!(&document.blocks()[0], Block::Paragraph(content) if matches!(content[0], Inline::Link { .. }) && matches!(content[2], Inline::Image { .. }))
        );
    }

    #[test]
    fn parses_gfm_tables() {
        let document = parse("| Name | Value |\n| --- | --- |\n| DocSail | TUI |");
        assert!(
            matches!(&document.blocks()[0], Block::Table { header, rows } if header.len() == 2 && rows.len() == 1)
        );
    }

    #[test]
    fn parses_links_and_images_in_table_cells() {
        let document =
            parse("| [DocSail](https://example.invalid) | ![Logo](logo.png) |\n| --- | --- |");

        assert!(
            matches!(&document.blocks()[0], Block::Table { header, .. } if matches!(header[0][0], Inline::Link { .. }) && matches!(header[1][0], Inline::Image { .. }))
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
