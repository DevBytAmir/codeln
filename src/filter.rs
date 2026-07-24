//! Decides which files and directories the walk in `counter` should visit,
//! based on the extension/name rules resolved into `Config`.

use crate::config::Config;
use std::path::Path;

/// Extension and name based include/exclude rules, resolved once from `Config`.
pub struct FileFilter {
    include_ext: Vec<String>,
    exclude_ext: Vec<String>,
    exclude_dirs: Vec<String>,
    exclude_files: Vec<String>,
}

impl FileFilter {
    pub fn new(config: &Config) -> Self {
        Self {
            include_ext: config.include_ext.clone(),
            exclude_ext: config.exclude_ext.clone(),
            exclude_dirs: config.exclude_dirs.clone(),
            exclude_files: config.exclude_files.clone(),
        }
    }

    /// Whether a file should be counted. `include_ext`, when non-empty, is an
    /// allow-list that takes priority over `exclude_ext`.
    pub fn should_process_file(&self, path: &Path) -> bool {
        let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
            return false;
        };

        if self.exclude_files.iter().any(|f| file_name == f) {
            return false;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        if !self.include_ext.is_empty() {
            return self.include_ext.iter().any(|e| e == ext);
        }

        !self.exclude_ext.iter().any(|e| e == ext)
    }

    /// Whether the walker should descend into this directory at all.
    pub fn should_process_dir(&self, path: &Path) -> bool {
        let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) else {
            return true;
        };

        !self.exclude_dirs.iter().any(|d| dir_name == d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn filter_with(
        include_ext: Vec<&str>,
        exclude_ext: Vec<&str>,
        exclude_dirs: Vec<&str>,
        exclude_files: Vec<&str>,
    ) -> FileFilter {
        let config = Config {
            include_ext: include_ext.into_iter().map(String::from).collect(),
            exclude_ext: exclude_ext.into_iter().map(String::from).collect(),
            exclude_dirs: exclude_dirs.into_iter().map(String::from).collect(),
            exclude_files: exclude_files.into_iter().map(String::from).collect(),
            ..Config::default()
        };
        FileFilter::new(&config)
    }

    #[test]
    fn exclude_ext_rejects_matching_files() {
        let filter = filter_with(vec![], vec!["log"], vec![], vec![]);
        assert!(!filter.should_process_file(&PathBuf::from("app.log")));
        assert!(filter.should_process_file(&PathBuf::from("app.rs")));
    }

    #[test]
    fn include_ext_is_an_allow_list_that_overrides_exclude_ext() {
        let filter = filter_with(vec!["rs"], vec!["rs"], vec![], vec![]);
        assert!(filter.should_process_file(&PathBuf::from("app.rs")));
        assert!(!filter.should_process_file(&PathBuf::from("app.py")));
    }

    #[test]
    fn exclude_files_matches_by_exact_name_not_extension() {
        let filter = filter_with(vec![], vec![], vec![], vec!["Cargo.lock"]);
        assert!(!filter.should_process_file(&PathBuf::from("Cargo.lock")));
        assert!(filter.should_process_file(&PathBuf::from("other.lock")));
    }

    #[test]
    fn exclude_dirs_matches_by_directory_name() {
        let filter = filter_with(vec![], vec![], vec!["node_modules"], vec![]);
        assert!(!filter.should_process_dir(&PathBuf::from("/repo/node_modules")));
        assert!(filter.should_process_dir(&PathBuf::from("/repo/src")));
    }

    #[test]
    fn extensionless_files_are_processed_when_not_explicitly_excluded() {
        let filter = filter_with(vec![], vec![], vec![], vec![]);
        assert!(filter.should_process_file(&PathBuf::from("Makefile")));
    }
}
