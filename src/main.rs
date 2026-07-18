use std::{env, ffi::OsString, path::PathBuf, process::ExitCode};

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
        Ok(Command::Run { path: _ }) => ExitCode::SUCCESS,
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

#[cfg(test)]
mod tests {
    use super::{Command, parse_command};
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
}
