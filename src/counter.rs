//! Walks the target directory and produces line/language statistics for
//! files that pass the `FileFilter`.

use crate::config::Config;
use crate::filter::FileFilter;
use crate::languages::get_language_name;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Aggregate line counts and per-file/per-language breakdowns for one run.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CountResults {
    pub total_files: usize,
    pub total_lines: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
    pub code_lines: usize,
    /// Per-file counts, keyed by relative path. Only populated when
    /// `show_details` is set. A `BTreeMap` for deterministic output.
    pub file_counts: BTreeMap<String, FileCount>,
    /// Files and pruned directories skipped by the filter, relative to
    /// the root. Only populated when `show_excluded` is set.
    pub excluded_files: Vec<String>,
    pub language_stats: BTreeMap<String, LanguageStats>,
}

/// Line counts for a single file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCount {
    pub total: usize,
    pub blank: usize,
    pub comments: usize,
    pub code: usize,
}

/// Aggregate stats for one detected language, as a share of total code lines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    pub files: usize,
    pub lines: usize,
    pub code_lines: usize,
    /// This language's share of `CountResults::code_lines`, 0-100.
    pub percentage: f64,
}

/// Renders `path` relative to `root`, falling back to the file name when
/// they're equal (root is a single file).
fn relative_display(path: &Path, root: &Path) -> String {
    let stripped = path.strip_prefix(root).unwrap_or(path);
    if stripped.as_os_str().is_empty() {
        path.file_name()
            .map_or_else(|| path.display().to_string(), |n| n.display().to_string())
    } else {
        stripped.display().to_string()
    }
}

/// Walks `root`, applying `filter`, and counts lines in every file that
/// passes. Unreadable or non-UTF-8 files are skipped, not fatal.
// Currently infallible (all fs errors are caught and skipped), but kept as
// `Result` since it's the natural place to propagate a walk-level failure
// later without changing every caller's signature.
#[allow(clippy::unnecessary_wraps)]
pub fn count_lines(root: &Path, filter: &FileFilter, config: &Config) -> Result<CountResults> {
    let mut results = CountResults::default();
    let mut language_files: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
    let mut excluded_dirs: Vec<String> = Vec::new();

    let walker = WalkDir::new(root)
        .follow_links(config.follow_links)
        .max_depth(config.max_depth.unwrap_or(usize::MAX))
        .into_iter()
        .filter_entry(|e| {
            // Never prune the root itself, or a root named e.g. "vendor"
            // would return zero results.
            if e.depth() == 0 {
                return true;
            }
            if e.file_type().is_dir() {
                let keep = filter.should_process_dir(e.path());
                if !keep && config.show_excluded {
                    excluded_dirs.push(relative_display(e.path(), root));
                }
                keep
            } else {
                true
            }
        });

    for entry in walker {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Warning: Failed to access path: {e}");
                continue;
            }
        };

        let path = entry.path();

        // Use the entry's own type, not a fresh stat, so --follow-links
        // applies to file symlinks too.
        if !entry.file_type().is_file() {
            continue;
        }

        if !filter.should_process_file(path) {
            if config.show_excluded {
                results.excluded_files.push(relative_display(path, root));
            }
            continue;
        }

        if let Ok(content) = fs::read_to_string(path) {
            let counts = count_file_lines(&content, config);

            results.total_files += 1;
            results.total_lines += counts.total;
            results.blank_lines += counts.blank;
            results.comment_lines += counts.comments;
            results.code_lines += counts.code;

            let lang = match path.extension().and_then(|e| e.to_str()) {
                Some(ext) => get_language_name(ext),
                None => "Other",
            };
            language_files
                .entry(lang.to_string())
                .or_default()
                .push((counts.total, counts.code));

            if config.show_details {
                results
                    .file_counts
                    .insert(relative_display(path, root), counts);
            }
        }
    }

    results.excluded_files.extend(excluded_dirs);
    results.excluded_files.sort();

    for (lang, files) in language_files {
        let file_count = files.len();
        let total_lines: usize = files.iter().map(|(total, _)| total).sum();
        let code_lines: usize = files.iter().map(|(_, code)| code).sum();
        // Line counts are nowhere near f64's 2^52 exact-integer limit, so
        // the precision loss clippy warns about here is not a real concern.
        #[allow(clippy::cast_precision_loss)]
        let percentage = if results.code_lines > 0 {
            (code_lines as f64 / results.code_lines as f64) * 100.0
        } else {
            0.0
        };

        results.language_stats.insert(
            lang,
            LanguageStats {
                files: file_count,
                lines: total_lines,
                code_lines,
                percentage,
            },
        );
    }

    Ok(results)
}

/// Classifies each line as blank, comment, or code. Comment detection is
/// prefix-based, not a real lexer. Disabling `count_blank`/`count_comments`
/// folds that category into `code` instead of tracking it separately.
fn count_file_lines(content: &str, config: &Config) -> FileCount {
    let mut total = 0;
    let mut blank = 0;
    let mut comments = 0;
    let mut code = 0;

    let mut in_block_comment = false;

    for line in content.lines() {
        total += 1;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            if config.count_blank {
                blank += 1;
            } else {
                code += 1;
            }
            continue;
        }

        if config.count_comments {
            if trimmed.starts_with("/*") || trimmed.starts_with("/**") {
                in_block_comment = true;
                comments += 1;
                if trimmed.contains("*/") {
                    in_block_comment = false;
                }
                continue;
            }

            if in_block_comment {
                comments += 1;
                if trimmed.contains("*/") {
                    in_block_comment = false;
                }
                continue;
            }

            if trimmed.starts_with("//")
                || trimmed.starts_with('#')
                || trimmed.starts_with("--")
                || trimmed.starts_with("<!--")
            {
                comments += 1;
                continue;
            }
        }

        code += 1;
    }

    FileCount {
        total,
        blank,
        comments,
        code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::FileFilter;
    use std::fs;

    fn config_with_show_details() -> Config {
        Config {
            show_details: true,
            ..Config::default()
        }
    }

    #[test]
    fn classifies_blank_comment_and_code_lines() {
        let content = "// header comment\nfn main() {\n\n    code();\n}\n";
        let counts = count_file_lines(content, &Config::default());
        assert_eq!(counts.total, 5);
        assert_eq!(counts.comments, 1);
        assert_eq!(counts.blank, 1);
        assert_eq!(counts.code, 3);
    }

    #[test]
    fn block_comments_span_multiple_lines() {
        let content = "/*\nstill a comment\n*/\ncode();\n";
        let counts = count_file_lines(content, &Config::default());
        assert_eq!(counts.comments, 3);
        assert_eq!(counts.code, 1);
    }

    #[test]
    fn disabling_comment_counting_reclassifies_comments_as_code() {
        let config = Config {
            count_comments: false,
            ..Config::default()
        };
        let content = "// not counted as a comment\ncode();\n";
        let counts = count_file_lines(content, &config);
        assert_eq!(counts.comments, 0);
        assert_eq!(counts.code, 2);
    }

    #[test]
    fn disabling_blank_counting_reclassifies_blanks_as_code() {
        let config = Config {
            count_blank: false,
            ..Config::default()
        };
        let content = "code();\n\n\ncode();\n";
        let counts = count_file_lines(content, &config);
        assert_eq!(counts.blank, 0);
        assert_eq!(counts.code, 4);
    }

    /// Builds a small, unique-per-test temp directory tree and runs
    /// `count_lines` against it end to end.
    fn in_temp_dir<F: FnOnce(&Path)>(name: &str, build: F) -> CountResults {
        let dir = std::env::temp_dir().join(format!("codeln_counter_test_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        build(&dir);

        let config = config_with_show_details();
        let filter = FileFilter::new(&config);
        let results = count_lines(&dir, &filter, &config).unwrap();

        fs::remove_dir_all(&dir).unwrap();
        results
    }

    #[test]
    fn extensionless_files_land_in_other_bucket_and_count_toward_totals() {
        let results = in_temp_dir("other_bucket", |dir| {
            fs::write(dir.join("a.rs"), "fn main() {}\n").unwrap();
            fs::write(dir.join("Makefile"), "all:\n\techo hi\n").unwrap();
        });

        assert_eq!(results.total_files, 2);
        assert!(results.language_stats.contains_key("Other"));
        assert!(results.language_stats.contains_key("Rust"));

        // Percentages must account for every counted line, including
        // extensionless files, so they sum to ~100%.
        let total_pct: f64 = results.language_stats.values().map(|s| s.percentage).sum();
        assert!((total_pct - 100.0).abs() < 0.01, "got {total_pct}");
    }

    #[test]
    fn excluded_directories_are_never_visited() {
        let results = in_temp_dir("exclude_dirs", |dir| {
            fs::write(dir.join("a.rs"), "fn main() {}\n").unwrap();
            fs::create_dir_all(dir.join("node_modules")).unwrap();
            fs::write(dir.join("node_modules/junk.js"), "console.log(1);\n").unwrap();
        });

        assert_eq!(results.total_files, 1);
        assert!(!results.file_counts.contains_key("node_modules/junk.js"));
    }

    #[test]
    fn analyzing_a_dir_whose_own_name_matches_an_exclude_entry_still_works() {
        // "vendor" is one of Config::default()'s excluded directory names.
        // The root itself must never be pruned by that rule, only its
        // descendants.
        let dir = std::env::temp_dir().join("codeln_counter_test_root_self_exclude/vendor");
        let parent = dir.parent().unwrap().to_path_buf();
        let _ = fs::remove_dir_all(&parent);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("a.rs"), "fn main() {}\n").unwrap();

        let config = config_with_show_details();
        let filter = FileFilter::new(&config);
        let results = count_lines(&dir, &filter, &config).unwrap();

        fs::remove_dir_all(&parent).unwrap();

        assert_eq!(
            results.total_files, 1,
            "root named 'vendor' must still be walked"
        );
    }

    #[test]
    fn show_excluded_reports_pruned_directories_not_just_files() {
        let dir = std::env::temp_dir().join("codeln_counter_test_show_excluded_dirs");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("a.rs"), "fn main() {}\n").unwrap();
        fs::create_dir_all(dir.join("node_modules")).unwrap();
        fs::write(dir.join("node_modules/junk.js"), "console.log(1);\n").unwrap();

        let mut config = config_with_show_details();
        config.show_excluded = true;
        let filter = FileFilter::new(&config);
        let results = count_lines(&dir, &filter, &config).unwrap();

        fs::remove_dir_all(&dir).unwrap();

        assert_eq!(results.total_files, 1);
        assert!(results.excluded_files.contains(&"node_modules".to_string()));
    }

    #[test]
    fn analyzing_a_single_file_path_keeps_its_filename() {
        let dir = std::env::temp_dir().join("codeln_counter_test_single_file");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("top.rs");
        fs::write(&file_path, "fn main() {}\n").unwrap();

        let config = config_with_show_details();
        let filter = FileFilter::new(&config);
        let results = count_lines(&file_path, &filter, &config).unwrap();

        fs::remove_dir_all(&dir).unwrap();

        assert_eq!(results.total_files, 1);
        assert!(
            results.file_counts.contains_key("top.rs"),
            "expected 'top.rs' key, got {:?}",
            results.file_counts.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    #[cfg(unix)]
    fn follow_links_gates_symlinked_files_not_just_directories() {
        let dir = std::env::temp_dir().join("codeln_counter_test_symlinked_file");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("real.rs"), "fn main() {}\n").unwrap();
        std::os::unix::fs::symlink(dir.join("real.rs"), dir.join("link.rs")).unwrap();

        let config = config_with_show_details();
        let filter = FileFilter::new(&config);
        let without_follow = count_lines(&dir, &filter, &config).unwrap();

        let config_follow = Config {
            follow_links: true,
            ..config_with_show_details()
        };
        let filter_follow = FileFilter::new(&config_follow);
        let with_follow = count_lines(&dir, &filter_follow, &config_follow).unwrap();

        fs::remove_dir_all(&dir).unwrap();

        assert_eq!(
            without_follow.total_files, 1,
            "symlink must not be dereferenced by default"
        );
        assert_eq!(
            with_follow.total_files, 2,
            "symlink must be counted as its own file with --follow-links"
        );
    }
}
