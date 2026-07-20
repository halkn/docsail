use std::{
    env,
    ffi::OsString,
    fs, io,
    path::{Path, PathBuf},
    process::ExitCode,
};

pub mod app;
pub mod event;
pub mod markdown;
pub mod terminal;
pub mod ui;
pub mod workspace;

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Run { path: Option<PathBuf> },
    Help,
}

fn parse_command(arguments: impl IntoIterator<Item = OsString>) -> Result<Command, String> {
    let mut arguments = arguments.into_iter();
    let _program_name = arguments.next();

    let Some(argument) = arguments.next() else {
        return Ok(Command::Run { path: None });
    };

    if argument == "-h" || argument == "--help" {
        if arguments.next().is_some() {
            return Err("help option does not accept a path".to_owned());
        }

        return Ok(Command::Help);
    }

    if arguments.next().is_some() {
        return Err("expected at most one path argument".to_owned());
    }

    Ok(Command::Run {
        path: Some(PathBuf::from(argument)),
    })
}

fn print_usage() {
    println!("Usage: docsail [PATH]");
}

fn main() -> ExitCode {
    match parse_command(env::args_os()) {
        Ok(Command::Run { path }) => match run(path) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("docsail: {error}");
                ExitCode::FAILURE
            }
        },
        Ok(Command::Help) => {
            print_usage();
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("docsail: {error}");
            print_usage();
            ExitCode::from(2)
        }
    }
}

fn run(path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let current_directory = env::current_dir()?;
    let workspace = workspace::resolve(path.as_deref(), &current_directory)?;
    let markdown_files = workspace::discover_markdown_files(&workspace)?;
    let mut tree = workspace::FileTree::from_files(&workspace.tree_root(), markdown_files)?;
    let mut terminal = terminal::TerminalSession::enter()?;
    let mut app = app::App::new();
    app.set_file_count(tree.file_count());
    let mut event_source = event::CrosstermEventSource;
    let mut back_history = Vec::<Location>::new();
    let mut forward_history = Vec::<Location>::new();
    let mut last_page_query = String::new();

    let run_result = event::run(&mut app, &mut event_source, |app| {
        if let Ok(files) = workspace::discover_markdown_files(&workspace)
            && let Ok(refreshed_tree) =
                workspace::FileTree::from_files(&workspace.tree_root(), files)
            && refreshed_tree != tree
        {
            tree = refreshed_tree;
            app.set_file_count(tree.file_count());
        }
        if app.take_back_request()
            && let Some(location) = back_history.pop()
        {
            forward_history.push(Location::from_app(app));
            app.navigate_to(location.file_index, location.scroll);
        }
        if app.take_forward_request()
            && let Some(location) = forward_history.pop()
        {
            back_history.push(Location::from_app(app));
            app.navigate_to(location.file_index, location.scroll);
        }

        let mut document = selected_document(&tree, app.selected_file_index())?;
        if let Some(overlay) = app.take_overlay_submission() {
            match overlay {
                app::Overlay::Toc => {
                    if let Some(heading) = document.headings().get(app.selected_result()) {
                        visit(
                            app,
                            &mut back_history,
                            &mut forward_history,
                            app.selected_file_index(),
                            ui::preview_scroll_for_block(&document, heading.block_index),
                        );
                    }
                }
                app::Overlay::FileSearch => {
                    if let Some(index) = file_search_results(&tree, app.query())
                        .get(app.selected_result())
                        .copied()
                    {
                        visit(app, &mut back_history, &mut forward_history, index, 0);
                    }
                }
                app::Overlay::PageSearch => {
                    last_page_query = app.query().to_owned();
                    if let Some(block) = document
                        .search_blocks(app.query())
                        .get(app.selected_result())
                        .copied()
                    {
                        visit(
                            app,
                            &mut back_history,
                            &mut forward_history,
                            app.selected_file_index(),
                            ui::preview_scroll_for_block(&document, block),
                        );
                    }
                }
            }
            document = selected_document(&tree, app.selected_file_index())?;
        }
        let next_page_match_requested = app.take_next_page_match_request();
        let previous_page_match_requested = app.take_previous_page_match_request();
        if (next_page_match_requested || previous_page_match_requested)
            && let Some(block) = next_page_match(
                &document,
                &last_page_query,
                app.preview_scroll(),
                previous_page_match_requested,
            )
        {
            visit(
                app,
                &mut back_history,
                &mut forward_history,
                app.selected_file_index(),
                ui::preview_scroll_for_block(&document, block),
            );
        }
        let files = tree_files(&tree);
        if app.take_activation_request()
            && let Some(source) = tree.file_at(app.selected_file_index())
            && let Some(target) = document
                .link_destinations()
                .into_iter()
                .find_map(|destination| {
                    workspace::resolve_markdown_link(
                        &workspace.tree_root(),
                        source,
                        destination,
                        &files,
                    )
                })
            && let Some(index) = tree_file_index(&tree, &target.path)
        {
            let target_document = fs::read_to_string(&target.path)
                .map(|source| markdown::parse(&source))
                .unwrap_or_default();
            let headings = target_document.headings();
            let scroll = target
                .anchor
                .as_ref()
                .and_then(|anchor| headings.iter().find(|heading| &heading.id == anchor))
                .map_or(0, |heading| {
                    ui::preview_scroll_for_block(&target_document, heading.block_index)
                });
            visit(app, &mut back_history, &mut forward_history, index, scroll);
            document = target_document;
        }
        let overlay_results = overlay_results(&tree, &document, app.overlay(), app.query());
        app.set_result_count(overlay_results.len());
        terminal
            .terminal_mut()
            .draw(|frame| {
                ui::render(
                    frame,
                    &tree,
                    ui::RenderState {
                        selected_file_index: app.selected_file_index(),
                        focus: app.focus(),
                        preview_scroll: app.preview_scroll(),
                        document: &document,
                        overlay: app.overlay(),
                        query: app.query(),
                        query_cursor: app.query_cursor(),
                        overlay_results: &overlay_results,
                        selected_result: app.selected_result(),
                    },
                )
            })
            .map(|_| ())
    });
    let restore_result = terminal.restore();

    Ok(run_result.and(restore_result)?)
}

#[derive(Clone, Copy)]
struct Location {
    file_index: usize,
    scroll: usize,
}
impl Location {
    fn from_app(app: &app::App) -> Self {
        Self {
            file_index: app.selected_file_index(),
            scroll: app.preview_scroll(),
        }
    }
}

fn visit(
    app: &mut app::App,
    back: &mut Vec<Location>,
    forward: &mut Vec<Location>,
    file_index: usize,
    scroll: usize,
) {
    let current = Location::from_app(app);
    if current.file_index != file_index || current.scroll != scroll {
        back.push(current);
        forward.clear();
        app.navigate_to(file_index, scroll);
    }
}

fn tree_files(tree: &workspace::FileTree) -> Vec<PathBuf> {
    (0..tree.file_count())
        .filter_map(|index| tree.file_at(index).map(Path::to_path_buf))
        .collect()
}
fn tree_file_index(tree: &workspace::FileTree, path: &Path) -> Option<usize> {
    (0..tree.file_count()).find(|index| tree.file_at(*index) == Some(path))
}

fn overlay_results(
    tree: &workspace::FileTree,
    document: &markdown::Document,
    overlay: Option<app::Overlay>,
    query: &str,
) -> Vec<String> {
    match overlay {
        Some(app::Overlay::Toc) => document
            .headings()
            .into_iter()
            .map(|heading| {
                format!(
                    "{}{}",
                    "  ".repeat(match heading.level {
                        markdown::HeadingLevel::One => 0,
                        markdown::HeadingLevel::Two => 1,
                        markdown::HeadingLevel::Three => 2,
                        markdown::HeadingLevel::Four => 3,
                        markdown::HeadingLevel::Five => 4,
                        markdown::HeadingLevel::Six => 5,
                    }),
                    heading.title
                )
            })
            .collect(),
        Some(app::Overlay::FileSearch) => file_search_results(tree, query)
            .into_iter()
            .filter_map(|index| tree.file_at(index).map(|path| relative_path(tree, path)))
            .collect(),
        Some(app::Overlay::PageSearch) => document
            .search_blocks(query)
            .into_iter()
            .filter_map(|index| document.block_text(index))
            .map(|text| search_result_label(&text))
            .collect(),
        None => Vec::new(),
    }
}

fn file_search_results(tree: &workspace::FileTree, query: &str) -> Vec<usize> {
    let query = query.to_lowercase();
    let mut matches = (0..tree.file_count())
        .filter_map(|index| {
            let path = tree.file_at(index)?;
            fuzzy_score(&relative_path(tree, path).to_lowercase(), &query)
                .map(|score| (score, index))
        })
        .collect::<Vec<_>>();
    matches.sort();
    matches.into_iter().map(|(_, index)| index).collect()
}

fn relative_path(tree: &workspace::FileTree, path: &Path) -> String {
    path.strip_prefix(tree.root().path())
        .unwrap_or(path)
        .display()
        .to_string()
}

fn search_result_label(text: &str) -> String {
    const MAX_CHARACTERS: usize = 72;
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut characters = normalized.chars();
    let label = characters.by_ref().take(MAX_CHARACTERS).collect::<String>();
    if characters.next().is_some() {
        format!("{label}…")
    } else {
        label
    }
}

fn fuzzy_score(value: &str, query: &str) -> Option<usize> {
    if query.is_empty() {
        return Some(usize::MAX / 2);
    }
    let mut offset = 0;
    let mut score = 0;
    for character in query.chars() {
        let found = value[offset..].find(character)?;
        score += found;
        offset += found + character.len_utf8();
    }
    Some(score)
}

fn next_page_match(
    document: &markdown::Document,
    query: &str,
    current_scroll: usize,
    previous: bool,
) -> Option<usize> {
    let matches = document.search_blocks(query);
    if previous {
        matches
            .iter()
            .rev()
            .find(|&&block| ui::preview_scroll_for_block(document, block) < current_scroll)
            .copied()
            .or_else(|| matches.last().copied())
    } else {
        matches
            .iter()
            .find(|&&block| ui::preview_scroll_for_block(document, block) > current_scroll)
            .copied()
            .or_else(|| matches.first().copied())
    }
}

fn selected_document(
    tree: &workspace::FileTree,
    selected_file_index: usize,
) -> io::Result<markdown::Document> {
    tree.file_at(selected_file_index)
        .map(fs::read_to_string)
        .transpose()
        .map(|source| {
            source.map_or_else(markdown::Document::default, |source| {
                markdown::parse(&source)
            })
        })
}

#[cfg(test)]
mod tests {
    use super::{Command, fuzzy_score, next_page_match, parse_command, search_result_label};
    use crate::markdown::parse;
    use std::{ffi::OsString, path::PathBuf};

    fn arguments(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn accepts_no_path() {
        assert_eq!(
            parse_command(arguments(&["docsail"])),
            Ok(Command::Run { path: None })
        );
    }

    #[test]
    fn accepts_one_path() {
        assert_eq!(
            parse_command(arguments(&["docsail", "docs"])),
            Ok(Command::Run {
                path: Some(PathBuf::from("docs"))
            })
        );
    }

    #[test]
    fn accepts_help() {
        assert_eq!(
            parse_command(arguments(&["docsail", "--help"])),
            Ok(Command::Help)
        );
    }

    #[test]
    fn rejects_multiple_paths() {
        assert_eq!(
            parse_command(arguments(&["docsail", "docs", "notes"])),
            Err("expected at most one path argument".to_owned())
        );
    }

    #[test]
    fn cycles_page_search_matches() {
        let document = parse("one needle\n\nsecond needle");

        assert_eq!(next_page_match(&document, "needle", 0, false), Some(1));
        assert_eq!(next_page_match(&document, "needle", 1, true), Some(0));
        assert_eq!(fuzzy_score("guide/setup.md", "gs"), Some(5));
    }

    #[test]
    fn labels_page_search_results_with_the_matching_text() {
        assert_eq!(
            search_result_label("first\n  matching text"),
            "first matching text"
        );
        assert_eq!(
            search_result_label(&"a".repeat(73)),
            format!("{}…", "a".repeat(72))
        );
    }
}
