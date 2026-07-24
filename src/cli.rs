use clap::{Parser, ValueHint};
use std::path::PathBuf;

/// `CodeLn` - A professional source code line counter with advanced filtering capabilities.
///
/// `CodeLn` analyzes your codebase and provides detailed statistics about lines of code,
/// including total lines, blank lines, comments, and pure code lines. It supports
/// flexible filtering through both configuration files and command-line arguments.
///
/// Examples:
///   codeln                                   # Analyze current directory
///   codeln ./src --show-details              # Analyze with per-file breakdown
///   codeln -c codeln.toml                    # Use configuration file
///   codeln -i rs,py -x txt,md                # Filter by extensions
///   codeln -d target,build --max-depth 5     # Exclude directories with depth limit
#[derive(Parser, Debug)]
#[command(name = "codeln")]
#[command(author = "DevBytAmir")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Professional line counter for code projects")]
#[command(
    long_about = "CodeLn is a professional source code line counter that provides \
    detailed statistics about your codebase. It supports flexible filtering through both \
    configuration files and command-line arguments, making it suitable for projects of any size."
)]
#[command(arg_required_else_help = false)]
// A CLI args struct is inherently a pile of independent on/off flags; splitting
// them into sub-structs would hurt clap ergonomics for no real benefit.
#[allow(clippy::struct_excessive_bools)]
pub struct Args {
    /// Path to the directory or file to analyze.
    ///
    /// Defaults to the config file's `path`, or the current working
    /// directory if unset.
    #[arg(value_name = "PATH", value_hint = ValueHint::DirPath, help_heading = "General")]
    pub path: Option<PathBuf>,

    /// Path to a TOML configuration file.
    ///
    /// CLI flags override settings in the configuration file.
    #[arg(short = 'c', long, value_name = "FILE", help_heading = "General")]
    pub config: Option<PathBuf>,

    /// Only include files with these specific extensions (comma-separated).
    #[arg(
        short = 'i',
        long = "include-ext",
        value_delimiter = ',',
        value_name = "EXTENSIONS",
        help_heading = "Filtering"
    )]
    pub include_ext: Option<Vec<String>>,

    /// Exclude files with these extensions (comma-separated).
    #[arg(
        short = 'x',
        long = "exclude-ext",
        value_delimiter = ',',
        value_name = "EXTENSIONS",
        help_heading = "Filtering"
    )]
    pub exclude_ext: Option<Vec<String>>,

    /// Directories to exclude (comma-separated).
    #[arg(
        short = 'd',
        long = "exclude-dirs",
        value_delimiter = ',',
        value_name = "DIRECTORIES",
        help_heading = "Filtering"
    )]
    pub exclude_dirs: Option<Vec<String>>,

    /// Specific files to exclude (comma-separated).
    #[arg(
        short = 'f',
        long = "exclude-files",
        value_delimiter = ',',
        value_name = "FILES",
        help_heading = "Filtering"
    )]
    pub exclude_files: Option<Vec<String>>,

    /// Exclude blank lines from statistics.
    #[arg(long, help_heading = "Counting")]
    pub no_count_blank: bool,

    /// Exclude comment lines from statistics.
    #[arg(long, help_heading = "Counting")]
    pub no_count_comments: bool,

    /// Follow symbolic links during traversal.
    #[arg(long, help_heading = "Traversal")]
    pub follow_links: bool,

    /// Maximum directory depth to traverse.
    #[arg(long, value_name = "DEPTH", help_heading = "Traversal")]
    pub max_depth: Option<usize>,

    /// Display line counts for each individual file.
    #[arg(long, help_heading = "Output")]
    pub show_details: bool,

    /// Display list of files and directories that were excluded.
    #[arg(long, help_heading = "Output")]
    pub show_excluded: bool,

    /// Output format (text, json, csv).
    ///
    /// Defaults to the config file's `format`, or "text" if unset.
    #[arg(long, value_name = "FORMAT", value_parser = ["text", "json", "csv"], help_heading = "Output")]
    pub format: Option<String>,

    /// Generate a default configuration file (`codeln.toml`) in the current directory.
    ///
    /// Prompts for confirmation if the file already exists.
    #[arg(long, help_heading = "Configuration")]
    pub init: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_path_or_format_override_by_default() {
        let args = Args::try_parse_from(["codeln"]).unwrap();
        assert_eq!(args.path, None);
        assert_eq!(args.format, None);
    }

    #[test]
    fn explicit_path_is_captured() {
        let args = Args::try_parse_from(["codeln", "src"]).unwrap();
        assert_eq!(args.path, Some(PathBuf::from("src")));
    }

    #[test]
    fn accepts_valid_format_values() {
        for fmt in ["text", "json", "csv"] {
            let args = Args::try_parse_from(["codeln", "--format", fmt]).unwrap();
            assert_eq!(args.format, Some(fmt.to_string()));
        }
    }

    #[test]
    fn rejects_invalid_format_value() {
        assert!(Args::try_parse_from(["codeln", "--format", "xml"]).is_err());
    }

    #[test]
    fn comma_separated_extensions_are_split() {
        let args = Args::try_parse_from(["codeln", "-i", "rs,py,go"]).unwrap();
        assert_eq!(
            args.include_ext,
            Some(vec!["rs".to_string(), "py".to_string(), "go".to_string()])
        );
    }
}
