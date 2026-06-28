//! Command-line argument parsing for the `bsv` binary.
//!
//! Kept separate from `main.rs` so the parsing logic is unit-testable without a
//! terminal. The interactive TUI is the default; `--validate` and `--json`
//! provide non-interactive modes suitable for CI.

use std::path::PathBuf;

/// A parsed invocation of the `bsv` binary.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    /// Print help and exit.
    Help,
    /// Print version and exit.
    Version,
    /// Launch the interactive TUI for the given path (or the default).
    Run(Option<PathBuf>),
    /// Validate the catalog and print a report; exit non-zero on errors.
    Validate(Option<PathBuf>),
    /// Print the parsed catalog as JSON.
    Json(Option<PathBuf>),
    /// An unrecognized option was supplied.
    Unknown(String),
}

/// Parse process arguments (including the program name at index 0).
pub fn parse_args(args: &[String]) -> Command {
    let mut rest = args.iter().skip(1);
    match rest.next().map(String::as_str) {
        Some("-h" | "--help") => Command::Help,
        Some("-V" | "--version") => Command::Version,
        Some("--validate") => Command::Validate(rest.next().map(PathBuf::from)),
        Some("--json") => Command::Json(rest.next().map(PathBuf::from)),
        // Any other leading-dash token is an unknown option, not a path.
        Some(opt) if opt.starts_with('-') => Command::Unknown(opt.to_string()),
        Some(path) => Command::Run(Some(PathBuf::from(path))),
        None => Command::Run(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(parts: &[&str]) -> Vec<String> {
        std::iter::once("bsv")
            .chain(parts.iter().copied())
            .map(String::from)
            .collect()
    }

    #[test]
    fn no_args_runs_default() {
        assert_eq!(parse_args(&args(&[])), Command::Run(None));
    }

    #[test]
    fn path_arg_runs_that_path() {
        assert_eq!(
            parse_args(&args(&["./catalog"])),
            Command::Run(Some(PathBuf::from("./catalog")))
        );
    }

    #[test]
    fn help_and_version_flags() {
        assert_eq!(parse_args(&args(&["-h"])), Command::Help);
        assert_eq!(parse_args(&args(&["--help"])), Command::Help);
        assert_eq!(parse_args(&args(&["-V"])), Command::Version);
        assert_eq!(parse_args(&args(&["--version"])), Command::Version);
    }

    #[test]
    fn validate_and_json_take_optional_path() {
        assert_eq!(parse_args(&args(&["--validate"])), Command::Validate(None));
        assert_eq!(
            parse_args(&args(&["--validate", "dir"])),
            Command::Validate(Some(PathBuf::from("dir")))
        );
        assert_eq!(parse_args(&args(&["--json"])), Command::Json(None));
        assert_eq!(
            parse_args(&args(&["--json", "f.yaml"])),
            Command::Json(Some(PathBuf::from("f.yaml")))
        );
    }

    #[test]
    fn unknown_option_is_flagged() {
        assert_eq!(
            parse_args(&args(&["--nope"])),
            Command::Unknown("--nope".to_string())
        );
    }
}
