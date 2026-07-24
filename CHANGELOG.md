# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-07-24

Initial release.

### Added

- Directory walk with line counting: total, blank, comment, and code lines,
  per file and aggregated per detected language.
- Output formats: `text` (colorized summary), `json`, and `csv`.
- Filtering via `-i/--include-ext`, `-x/--exclude-ext`, `-d/--exclude-dirs`,
  `-f/--exclude-files`, and `--max-depth`.
- Configuration via an optional TOML file (`-c/--config`), merged with CLI
  flags (CLI always takes precedence over the config file).
- `--init` to generate a documented `codeln.toml` template in the current
  directory.
- `--show-details` (per-file breakdown) and `--show-excluded` (files and
  directories skipped by the active filters, including directories pruned
  entirely from the walk).
- `--no-count-blank` / `--no-count-comments` to fold those line categories
  into the code count instead of tracking them separately, consistently
  across every output format.
- Sensible built-in default excludes for common VCS/build/dependency
  directories (`.git`, `target`, `node_modules`, `vendor`, etc.).
- Symlink handling via `--follow-links`, applied consistently to both
  symlinked files and symlinked directories during traversal.
- Deterministic, sorted output order for per-file and per-language
  results across repeated runs.
