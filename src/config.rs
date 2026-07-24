//! Resolved run configuration: loaded from an optional TOML file, then
//! overridden by CLI flags.

use crate::cli::Args;
use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Effective settings for one run, after merging config file and CLI args.
// Mirrors `Args`: a config struct legitimately has many independent toggles.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_path")]
    pub path: PathBuf,

    #[serde(default)]
    pub include_ext: Vec<String>,

    #[serde(default = "default_exclude_ext")]
    pub exclude_ext: Vec<String>,

    #[serde(default = "default_exclude_dirs")]
    pub exclude_dirs: Vec<String>,

    #[serde(default)]
    pub exclude_files: Vec<String>,

    #[serde(default)]
    pub show_details: bool,

    #[serde(default)]
    pub show_excluded: bool,

    #[serde(default = "default_true")]
    pub count_blank: bool,

    #[serde(default = "default_true")]
    pub count_comments: bool,

    #[serde(default)]
    pub follow_links: bool,

    #[serde(default)]
    pub max_depth: Option<usize>,

    #[serde(default = "default_format")]
    pub format: String,
}

fn default_path() -> PathBuf {
    PathBuf::from(".")
}

fn default_true() -> bool {
    true
}

fn default_format() -> String {
    "text".to_string()
}

fn default_exclude_ext() -> Vec<String> {
    vec![]
}

/// Strip a leading dot from each extension so `".rs"` and `"rs"` are
/// equivalent, regardless of whether they came from the CLI or a TOML file.
fn strip_leading_dots(extensions: &[String]) -> Vec<String> {
    extensions
        .iter()
        .map(|e| e.trim_start_matches('.').to_string())
        .collect()
}

const VALID_FORMATS: [&str; 3] = ["text", "json", "csv"];

fn default_exclude_dirs() -> Vec<String> {
    vec![
        ".git".to_string(),
        ".svn".to_string(),
        ".hg".to_string(),
        "target".to_string(),
        "node_modules".to_string(),
        "dist".to_string(),
        "build".to_string(),
        ".idea".to_string(),
        ".vscode".to_string(),
        "__pycache__".to_string(),
        ".pytest_cache".to_string(),
        "vendor".to_string(),
    ]
}

impl Config {
    /// Loads the config file (if any), then applies CLI overrides on top.
    /// Excludes are additive; everything else is CLI-wins.
    pub fn load(args: &Args) -> Result<Self> {
        let mut config = if let Some(config_path) = &args.config {
            let content = fs::read_to_string(config_path).context(format!(
                "Failed to read config file: {}",
                config_path.display()
            ))?;
            toml::from_str(&content).context("Failed to parse TOML config file")?
        } else {
            Self::default()
        };

        config.include_ext = strip_leading_dots(&config.include_ext);
        config.exclude_ext = strip_leading_dots(&config.exclude_ext);

        if let Some(ref path) = args.path {
            path.clone_into(&mut config.path);
        }

        if !config.path.exists() {
            return Err(anyhow!("Path not found: {}", config.path.display()));
        }

        if let Some(ref ext) = args.include_ext {
            config.include_ext = strip_leading_dots(ext);
        }

        if let Some(ref ext) = args.exclude_ext {
            config.exclude_ext.extend(strip_leading_dots(ext));
        }

        if let Some(ref dirs) = args.exclude_dirs {
            config.exclude_dirs.extend(dirs.clone());
        }

        if let Some(ref files) = args.exclude_files {
            config.exclude_files.extend(files.clone());
        }

        if args.show_details {
            config.show_details = true;
        }

        if args.show_excluded {
            config.show_excluded = true;
        }

        if args.no_count_blank {
            config.count_blank = false;
        }

        if args.no_count_comments {
            config.count_comments = false;
        }

        if args.follow_links {
            config.follow_links = true;
        }

        if args.max_depth.is_some() {
            config.max_depth = args.max_depth;
        }

        if let Some(ref format) = args.format {
            format.clone_into(&mut config.format);
        }

        if !VALID_FORMATS.contains(&config.format.as_str()) {
            return Err(anyhow!(
                "Invalid format '{}' in config file (expected one of: {})",
                config.format,
                VALID_FORMATS.join(", ")
            ));
        }

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            path: default_path(),
            include_ext: vec![],
            exclude_ext: default_exclude_ext(),
            exclude_dirs: default_exclude_dirs(),
            exclude_files: vec![],
            show_details: false,
            show_excluded: false,
            count_blank: true,
            count_comments: true,
            follow_links: false,
            max_depth: None,
            format: default_format(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn base_args() -> Args {
        Args {
            path: None,
            config: None,
            include_ext: None,
            exclude_ext: None,
            exclude_dirs: None,
            exclude_files: None,
            no_count_blank: false,
            no_count_comments: false,
            follow_links: false,
            max_depth: None,
            show_details: false,
            show_excluded: false,
            format: None,
            init: false,
        }
    }

    #[test]
    fn default_config_matches_documented_defaults() {
        let config = Config::default();
        assert!(config.count_blank);
        assert!(config.count_comments);
        assert!(!config.show_details);
        assert!(!config.show_excluded);
        assert_eq!(config.format, "text");
        assert!(config.exclude_dirs.contains(&"node_modules".to_string()));
    }

    #[test]
    fn missing_path_is_an_error() {
        let mut args = base_args();
        args.path = Some(PathBuf::from("/this/path/does/not/exist/hopefully"));
        assert!(Config::load(&args).is_err());
    }

    #[test]
    fn cli_path_overrides_default() {
        let mut args = base_args();
        args.path = Some(PathBuf::from("src"));
        let config = Config::load(&args).unwrap();
        assert_eq!(config.path, PathBuf::from("src"));
    }

    #[test]
    fn config_file_path_is_used_when_cli_omits_it() {
        let dir = std::env::temp_dir().join("codeln_config_test_path_from_file");
        let _ = fs::remove_dir_all(&dir);
        let target = dir.join("onlyme");
        fs::create_dir_all(&target).unwrap();
        let config_file = dir.join("codeln.toml");
        // Absolute path so the test doesn't depend on the process's cwd,
        // which is shared global state across parallel test threads.
        fs::write(&config_file, format!("path = {target:?}\n")).unwrap();

        let mut args = base_args();
        args.config = Some(config_file);
        let config = Config::load(&args).unwrap();

        assert_eq!(config.path, target);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn invalid_format_in_config_file_is_rejected() {
        let mut args = base_args();
        let dir = std::env::temp_dir().join("codeln_config_test_bad_format");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let config_file = dir.join("codeln.toml");
        fs::write(&config_file, "format = \"yaml\"\n").unwrap();
        args.config = Some(config_file);

        let result = Config::load(&args);
        assert!(result.is_err());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn no_count_flags_turn_off_defaults() {
        let mut args = base_args();
        args.no_count_blank = true;
        args.no_count_comments = true;
        let config = Config::load(&args).unwrap();
        assert!(!config.count_blank);
        assert!(!config.count_comments);
    }

    #[test]
    fn include_ext_strips_leading_dot() {
        let mut args = base_args();
        args.include_ext = Some(vec![".rs".to_string(), "py".to_string()]);
        let config = Config::load(&args).unwrap();
        assert_eq!(config.include_ext, vec!["rs".to_string(), "py".to_string()]);
    }

    #[test]
    fn config_file_extensions_also_strip_leading_dot() {
        let dir = std::env::temp_dir().join("codeln_config_test_dotted_ext");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let config_file = dir.join("codeln.toml");
        fs::write(
            &config_file,
            "include_ext = [\".rs\", \"py\"]\nexclude_ext = [\".log\"]\n",
        )
        .unwrap();

        let mut args = base_args();
        args.config = Some(config_file);
        let config = Config::load(&args).unwrap();

        assert_eq!(config.include_ext, vec!["rs".to_string(), "py".to_string()]);
        assert!(config.exclude_ext.contains(&"log".to_string()));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn cli_excludes_are_additive_not_replacing() {
        let mut args = base_args();
        args.exclude_dirs = Some(vec!["custom_dir".to_string()]);
        let config = Config::load(&args).unwrap();
        // Built-in default excludes are still present alongside the CLI addition.
        assert!(config.exclude_dirs.contains(&"target".to_string()));
        assert!(config.exclude_dirs.contains(&"custom_dir".to_string()));
    }

    #[test]
    fn cli_format_overrides_default() {
        let mut args = base_args();
        args.format = Some("json".to_string());
        let config = Config::load(&args).unwrap();
        assert_eq!(config.format, "json");
    }

    #[test]
    fn format_defaults_to_text_without_cli_override() {
        let args = base_args();
        let config = Config::load(&args).unwrap();
        assert_eq!(config.format, "text");
    }
}
