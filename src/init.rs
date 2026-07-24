//! Generates the `codeln.toml` template written by `--init`.

use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Write `codeln.toml` in the current directory, prompting before overwrite
/// if one already exists.
pub fn generate_config_file() -> Result<()> {
    let config_path = Path::new("codeln.toml");

    if config_path.exists() {
        println!("{}", "Configuration file already exists.".yellow().bold());
        print!("Do you want to overwrite it? [y/N]: ");
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        let response = response.trim().to_lowercase();
        if response != "y" && response != "yes" {
            println!("{}", "Operation cancelled.".cyan());
            return Ok(());
        }
    }

    let config_content = generate_default_config_content();

    fs::write(config_path, config_content).context("Failed to write configuration file")?;

    println!(
        "{}",
        "Configuration file created successfully!".green().bold()
    );
    println!("  Created: {}", config_path.display().to_string().cyan());
    println!("\n{}", "Next steps:".bold());
    println!(
        "  1. Edit {} to customize your settings",
        "codeln.toml".cyan()
    );
    println!(
        "  2. Run {} to analyze your code",
        "codeln -c codeln.toml".cyan()
    );

    Ok(())
}

/// The commented `codeln.toml` template written by `generate_config_file`.
/// Kept in sync by hand with the `Config` fields and their defaults.
fn generate_default_config_content() -> String {
    r#"# CodeLn Configuration File
#
# This file configures the behavior of CodeLn, a professional source code
# line counter. All settings can be overridden via command-line arguments.

# Path to analyze (can be overridden by CLI argument)
# Defaults to current directory if not specified
path = "."

# Include only specific file extensions
# Leave empty to include all file types (recommended for most projects)
# Extensions should be specified without the leading dot
# Example: ["rs", "toml", "md"]
include_ext = []

# Exclude specific file extensions
# Nothing is excluded by extension unless you list it here or pass
# --exclude-ext; this is just a commonly-useful starting point.
# Extensions should be specified without the leading dot
exclude_ext = ["lock", "log", "tmp", "cache"]

# Exclude directories
# These directories will be completely skipped during analysis. Most of
# the list below is already CodeLn's built-in default (used even without
# a config file). "out", "venv", "env", and ".vs" are added here as
# commonly-useful extras.
exclude_dirs = [
    # Version control
    ".git",
    ".svn",
    ".hg",

    # Build outputs
    "target",
    "build",
    "dist",
    "out",

    # Dependencies
    "node_modules",
    "vendor",

    # Python
    "__pycache__",
    ".pytest_cache",
    "venv",
    "env",

    # IDE folders
    ".idea",
    ".vscode",
    ".vs",
]

# Exclude specific files by exact name
# Useful for generated files or lock files you want to skip
exclude_files = [
    "package-lock.json",
    "Cargo.lock",
    "yarn.lock",
    "pnpm-lock.yaml",
    "poetry.lock",
]

# Display Options
# ----------------

# Show detailed line counts for each individual file
# Useful for finding which files have the most code
show_details = false

# Show list of files that were excluded by filters
# Helpful for debugging filter configurations
show_excluded = false

# Counting Options
# ----------------

# Count blank lines in the statistics
# Blank lines are lines containing only whitespace
count_blank = true

# Count comment lines in the statistics
# Supports common comment styles: //, #, /* */, <!--
count_comments = true

# Traversal Options
# -----------------

# Follow symbolic links during directory traversal
# Warning: May cause infinite loops if links create cycles
follow_links = false

# Maximum directory depth to traverse
# Uncomment and set a value to limit recursion depth
# Depth 1 = only specified directory, 2 = one level down, etc.
# max_depth = 10

# Output Options
# --------------

# Default output format: "text", "json", or "csv"
# Overridden by the --format CLI flag
format = "text"
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn template_parses_into_config_and_matches_defaults() {
        let content = generate_default_config_content();
        let parsed: Config = toml::from_str(&content).expect("template must be valid TOML");
        let defaults = Config::default();

        assert_eq!(parsed.show_details, defaults.show_details);
        assert_eq!(parsed.show_excluded, defaults.show_excluded);
        assert_eq!(parsed.count_blank, defaults.count_blank);
        assert_eq!(parsed.count_comments, defaults.count_comments);
        assert_eq!(parsed.follow_links, defaults.follow_links);
        assert_eq!(parsed.format, defaults.format);
        assert_eq!(parsed.exclude_ext, vec!["lock", "log", "tmp", "cache"]);
    }
}
