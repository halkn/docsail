use pulldown_cmark::{
    Alignment, CodeBlockKind, Event, HeadingLevel as ParserHeadingLevel, LinkType, Options, Parser,
    Tag, TagEnd,
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

    pub fn headings(&self) -> Vec<Heading> {
        let mut occurrences = std::collections::BTreeMap::<String, usize>::new();
        self.blocks
            .iter()
            .enumerate()
            .filter_map(|(block_index, block)| match block {
                Block::Heading { level, content } => {
                    let title = inline_plain_text(content);
                    let base = heading_slug(&title);
                    let occurrence = occurrences.entry(base.clone()).or_default();
                    let id = if *occurrence == 0 {
                        base
                    } else {
                        format!("{base}-{}", *occurrence)
                    };
                    *occurrence += 1;
                    Some(Heading {
                        level: *level,
                        title,
                        id,
                        block_index,
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub fn search_blocks(&self, query: &str) -> Vec<usize> {
        let query = query.to_lowercase();
        if query.is_empty() {
            return Vec::new();
        }
        self.blocks
            .iter()
            .enumerate()
            .filter_map(|(index, block)| {
                block_plain_text(block)
                    .to_lowercase()
                    .contains(&query)
                    .then_some(index)
            })
            .collect()
    }

    pub fn block_text(&self, index: usize) -> Option<String> {
        self.blocks.get(index).map(block_plain_text)
    }

    pub fn link_destinations(&self) -> Vec<&str> {
        let mut destinations = Vec::new();
        for block in &self.blocks {
            collect_block_links(block, &mut destinations);
        }
        destinations
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Heading {
    pub level: HeadingLevel,
    pub title: String,
    pub id: String,
    pub block_index: usize,
}

pub fn heading_slug(title: &str) -> String {
    let slug = title
        .chars()
        .flat_map(char::to_lowercase)
        .filter_map(|character| {
            if character.is_alphanumeric() || matches!(character, '-' | '_' | '\u{3000}') {
                Some(if character == '\u{3000}' {
                    '-'
                } else {
                    character
                })
            } else if character.is_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>();
    slug.trim_matches('-').to_owned()
}

fn inline_plain_text(inlines: &[Inline]) -> String {
    inlines.iter().fold(String::new(), |mut text, inline| {
        match inline {
            Inline::Text(value)
            | Inline::Autolink(value)
            | Inline::Code(value)
            | Inline::Html(value) => text.push_str(value),
            Inline::Link { content, .. }
            | Inline::Image { alt: content, .. }
            | Inline::Emphasis(content)
            | Inline::Strong(content)
            | Inline::Strikethrough(content) => text.push_str(&inline_plain_text(content)),
            Inline::SoftBreak | Inline::HardBreak => text.push(' '),
        }
        text
    })
}

fn block_plain_text(block: &Block) -> String {
    match block {
        Block::Heading { content, .. } | Block::Paragraph(content) => inline_plain_text(content),
        Block::List { items, .. } => items
            .iter()
            .flat_map(|item| item.blocks.iter())
            .map(block_plain_text)
            .collect::<Vec<_>>()
            .join(" "),
        Block::BlockQuote(blocks) => blocks
            .iter()
            .map(block_plain_text)
            .collect::<Vec<_>>()
            .join(" "),
        Block::CodeBlock { content, .. } | Block::Html(content) => content.clone(),
        Block::Table { header, rows, .. } => header
            .iter()
            .chain(rows.iter().flatten())
            .map(|cell| inline_plain_text(cell))
            .collect::<Vec<_>>()
            .join(" "),
        Block::ThematicBreak => String::new(),
    }
}

fn collect_block_links<'a>(block: &'a Block, destinations: &mut Vec<&'a str>) {
    match block {
        Block::Heading { content, .. } | Block::Paragraph(content) => {
            collect_inline_links(content, destinations)
        }
        Block::List { items, .. } => {
            for item in items {
                for block in &item.blocks {
                    collect_block_links(block, destinations);
                }
            }
        }
        Block::BlockQuote(blocks) => {
            for block in blocks {
                collect_block_links(block, destinations);
            }
        }
        Block::Table { header, rows, .. } => {
            for cell in header.iter().chain(rows.iter().flatten()) {
                collect_inline_links(cell, destinations);
            }
        }
        Block::CodeBlock { .. } | Block::ThematicBreak | Block::Html(_) => {}
    }
}

fn collect_inline_links<'a>(inlines: &'a [Inline], destinations: &mut Vec<&'a str>) {
    for inline in inlines {
        match inline {
            Inline::Link {
                content,
                destination,
            } => {
                destinations.push(destination);
                collect_inline_links(content, destinations);
            }
            Inline::Image { alt, .. }
            | Inline::Emphasis(alt)
            | Inline::Strong(alt)
            | Inline::Strikethrough(alt) => collect_inline_links(alt, destinations),
            _ => {}
        }
    }
}

pub fn parse(source: &str) -> Document {
    let mut blocks = Vec::new();
    let mut active_block = None;
    let mut code_block = None;
    let mut lists = Vec::new();
    let mut quote_blocks = Vec::new();
    let mut table = None;
    let mut inline_spans = Vec::new();

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    for event in Parser::new_ext(source, options) {
        match event {
            Event::Start(Tag::Emphasis) => {
                start_inline_span(&mut inline_spans, &mut active_block, &mut table, |start| {
                    InlineSpan::Emphasis { start }
                });
            }
            Event::Start(Tag::Strong) => {
                start_inline_span(&mut inline_spans, &mut active_block, &mut table, |start| {
                    InlineSpan::Strong { start }
                });
            }
            Event::Start(Tag::Strikethrough) => {
                start_inline_span(&mut inline_spans, &mut active_block, &mut table, |start| {
                    InlineSpan::Strikethrough { start }
                });
            }
            Event::Start(Tag::Link {
                link_type: LinkType::Autolink | LinkType::Email,
                dest_url,
                ..
            }) => {
                start_inline_span(&mut inline_spans, &mut active_block, &mut table, |start| {
                    InlineSpan::Autolink {
                        start,
                        destination: dest_url.into_string(),
                    }
                });
            }
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
            Event::End(TagEnd::Emphasis)
            | Event::End(TagEnd::Strong)
            | Event::End(TagEnd::Strikethrough) => {
                if let Some(span) = inline_spans.pop()
                    && let Some(content) = active_content(&mut active_block, &mut table)
                {
                    span.wrap(content);
                }
            }
            Event::Start(Tag::Table(alignments)) => {
                table = Some(TableContext::new(alignments));
            }
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
                    push_block(
                        &mut blocks,
                        &mut lists,
                        &mut quote_blocks,
                        Block::Table {
                            header: table.header,
                            rows: table.rows,
                            alignments: table.alignments,
                        },
                    );
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
                        &mut lists,
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
                        &mut lists,
                        &mut quote_blocks,
                        Block::Paragraph(content),
                    );
                }
            }
            Event::Rule => push_block(
                &mut blocks,
                &mut lists,
                &mut quote_blocks,
                Block::ThematicBreak,
            ),
            Event::Start(Tag::List(start)) => {
                flush_active_block(
                    &mut active_block,
                    &mut blocks,
                    &mut lists,
                    &mut quote_blocks,
                );
                lists.push(ListContext::new(start));
            }
            Event::Start(Tag::Item) => {
                if let Some(list) = lists.last_mut() {
                    list.current = Some(ListItem {
                        task: None,
                        blocks: Vec::new(),
                    });
                    // pulldown-cmark omits paragraph events for tight list items.
                    active_block = Some(ActiveBlock::Paragraph(Vec::new()));
                }
            }
            Event::TaskListMarker(checked) => {
                if let Some(Some(item)) = lists.last_mut().map(|list| list.current.as_mut()) {
                    item.task = Some(checked);
                }
            }
            Event::End(TagEnd::Item) => {
                if let Some(ActiveBlock::Paragraph(content)) = active_block.take()
                    && let Some(Some(item)) = lists.last_mut().map(|list| list.current.as_mut())
                    && !content.is_empty()
                {
                    item.blocks.push(Block::Paragraph(content));
                }
                if let Some(list) = lists.last_mut()
                    && let Some(item) = list.current.take()
                {
                    list.items.push(item);
                }
            }
            Event::End(TagEnd::List(_)) => {
                if let Some(list) = lists.pop() {
                    push_block(
                        &mut blocks,
                        &mut lists,
                        &mut quote_blocks,
                        Block::List {
                            start: list.start,
                            items: list.items,
                        },
                    );
                }
            }
            Event::Start(Tag::BlockQuote(_)) => {
                flush_active_block(
                    &mut active_block,
                    &mut blocks,
                    &mut lists,
                    &mut quote_blocks,
                );
                quote_blocks.push(Vec::new());
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                if let Some(quoted) = quote_blocks.pop() {
                    push_block(
                        &mut blocks,
                        &mut lists,
                        &mut quote_blocks,
                        Block::BlockQuote(quoted),
                    );
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
                    push_block(
                        &mut blocks,
                        &mut lists,
                        &mut quote_blocks,
                        Block::CodeBlock { language, content },
                    );
                }
            }
            Event::Text(text) => {
                if let Some((_, content)) = &mut code_block {
                    content.push_str(&text);
                } else if let Some(Some(cell)) =
                    table.as_mut().map(|table| table.current_cell.as_mut())
                {
                    cell.push(Inline::Text(text.into_string()));
                } else {
                    push_inline(&mut active_block, Inline::Text(text.into_string()));
                }
            }
            Event::Code(code) => {
                if let Some(content) = active_content(&mut active_block, &mut table) {
                    content.push(Inline::Code(code.into_string()));
                }
            }
            Event::SoftBreak => push_inline(&mut active_block, Inline::SoftBreak),
            Event::HardBreak => push_inline(&mut active_block, Inline::HardBreak),
            Event::Html(html) => push_block(
                &mut blocks,
                &mut lists,
                &mut quote_blocks,
                Block::Html(html.into_string()),
            ),
            Event::InlineHtml(html) => {
                push_inline(&mut active_block, Inline::Html(html.into_string()))
            }
            _ => {}
        }
    }

    Document::new(blocks)
}

struct ListContext {
    start: Option<u64>,
    items: Vec<ListItem>,
    current: Option<ListItem>,
}
impl ListContext {
    fn new(start: Option<u64>) -> Self {
        Self {
            start,
            items: Vec::new(),
            current: None,
        }
    }
}

fn push_block(
    blocks: &mut Vec<Block>,
    lists: &mut [ListContext],
    quotes: &mut [Vec<Block>],
    block: Block,
) {
    if let Some(quoted) = quotes.last_mut() {
        quoted.push(block);
    } else if let Some(Some(item)) = lists.last_mut().map(|list| list.current.as_mut()) {
        item.blocks.push(block);
    } else {
        blocks.push(block);
    }
}

fn flush_active_block(
    active_block: &mut Option<ActiveBlock>,
    blocks: &mut Vec<Block>,
    lists: &mut [ListContext],
    quotes: &mut [Vec<Block>],
) {
    match active_block.take() {
        Some(ActiveBlock::Heading { level, content }) => {
            push_block(blocks, lists, quotes, Block::Heading { level, content });
        }
        Some(ActiveBlock::Paragraph(content)) if !content.is_empty() => {
            push_block(blocks, lists, quotes, Block::Paragraph(content));
        }
        Some(ActiveBlock::Paragraph(_)) | None => {}
    }
}

enum InlineSpan {
    Emphasis { start: usize },
    Strong { start: usize },
    Strikethrough { start: usize },
    Link { start: usize, destination: String },
    Image { start: usize, destination: String },
    Autolink { start: usize, destination: String },
}
impl InlineSpan {
    fn wrap(self, content: &mut Vec<Inline>) {
        let start = match &self {
            Self::Emphasis { start }
            | Self::Strong { start }
            | Self::Strikethrough { start }
            | Self::Link { start, .. }
            | Self::Image { start, .. }
            | Self::Autolink { start, .. } => *start,
        };
        let nested = content.split_off(start);
        content.push(match self {
            Self::Emphasis { .. } => Inline::Emphasis(nested),
            Self::Strong { .. } => Inline::Strong(nested),
            Self::Strikethrough { .. } => Inline::Strikethrough(nested),
            Self::Link { destination, .. } => Inline::Link {
                content: nested,
                destination,
            },
            Self::Image { destination, .. } => Inline::Image {
                alt: nested,
                destination,
            },
            Self::Autolink { destination, .. } => Inline::Autolink(destination),
        });
    }
}

fn start_inline_span(
    inline_spans: &mut Vec<InlineSpan>,
    active_block: &mut Option<ActiveBlock>,
    table: &mut Option<TableContext>,
    make_span: impl FnOnce(usize) -> InlineSpan,
) {
    if let Some(content) = active_content(active_block, table) {
        inline_spans.push(make_span(content.len()));
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

struct TableContext {
    alignments: Vec<TableAlignment>,
    header: Vec<Vec<Inline>>,
    rows: Vec<Vec<Vec<Inline>>>,
    current_row: Option<Vec<Vec<Inline>>>,
    current_cell: Option<Vec<Inline>>,
    in_header: bool,
}

impl TableContext {
    fn new(alignments: Vec<Alignment>) -> Self {
        Self {
            alignments: alignments.into_iter().map(TableAlignment::from).collect(),
            header: Vec::new(),
            rows: Vec::new(),
            current_row: None,
            current_cell: None,
            in_header: false,
        }
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
        start: Option<u64>,
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
        alignments: Vec<TableAlignment>,
    },
    ThematicBreak,
    Html(String),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableAlignment {
    None,
    Left,
    Center,
    Right,
}

impl From<Alignment> for TableAlignment {
    fn from(value: Alignment) -> Self {
        match value {
            Alignment::None => Self::None,
            Alignment::Left => Self::Left,
            Alignment::Center => Self::Center,
            Alignment::Right => Self::Right,
        }
    }
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
    Html(String),
}

#[cfg(test)]
mod tests {
    use super::{
        Block, Document, HeadingLevel, Inline, ListItem, TableAlignment, heading_slug, parse,
    };

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
    fn parses_lists_tasks_and_blockquotes() {
        let document = parse("- [x] done\n- next\n\n> quoted");
        assert!(
            matches!(&document.blocks()[0], Block::List { items, .. } if items[0].task == Some(true))
        );
        assert!(matches!(&document.blocks()[1], Block::BlockQuote(_)));
    }

    #[test]
    fn preserves_nested_lists_and_blockquotes() {
        let document = parse("- parent\n  - child\n- sibling\n\n> outer\n> > inner");

        assert!(matches!(&document.blocks()[0], Block::List { items, .. }
            if items.len() == 2
                && matches!(items[0].blocks.as_slice(), [Block::Paragraph(_), Block::List { items, .. }] if items.len() == 1)
                && matches!(items[1].blocks.as_slice(), [Block::Paragraph(_)])));
        assert!(matches!(&document.blocks()[1], Block::BlockQuote(blocks)
            if matches!(blocks.as_slice(), [Block::Paragraph(_), Block::BlockQuote(_)])));
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
            matches!(&document.blocks()[0], Block::Table { header, rows, alignments } if header.len() == 2 && rows.len() == 1 && alignments == &[TableAlignment::None, TableAlignment::None])
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
    fn parses_emphasis_strikethrough_autolinks_and_thematic_breaks() {
        let document = parse("*emphasis* **strong** ~~strike~~ <https://example.invalid>\n\n---");

        assert!(matches!(&document.blocks()[0], Block::Paragraph(content)
                if matches!(content[0], Inline::Emphasis(_))
                && matches!(content[2], Inline::Strong(_))
                && matches!(content[4], Inline::Strikethrough(_))
                && matches!(content[6], Inline::Autolink(_))));
        assert!(matches!(document.blocks()[1], Block::ThematicBreak));
    }

    #[test]
    fn preserves_blocks_and_nested_gfm_content_without_rendering_types() {
        let document = Document::new(vec![
            Block::Heading {
                level: HeadingLevel::One,
                content: vec![Inline::Text("DocSail".to_owned())],
            },
            Block::List {
                start: None,
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

    #[test]
    fn creates_github_style_heading_ids_and_searches_blocks() {
        let document = parse("# Hello, World!\n\ntext needle\n\n## Hello, World!");

        assert_eq!(heading_slug("日本語 Heading"), "日本語-heading");
        assert_eq!(
            document
                .headings()
                .into_iter()
                .map(|heading| heading.id)
                .collect::<Vec<_>>(),
            ["hello-world", "hello-world-1"]
        );
        assert_eq!(document.search_blocks("NEEDLE"), [1]);
        assert_eq!(document.block_text(1).as_deref(), Some("text needle"));
    }
}
