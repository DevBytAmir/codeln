# CodeLn

[![CI](https://github.com/DevBytAmir/codeln/actions/workflows/ci.yml/badge.svg)](https://github.com/DevBytAmir/codeln/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/DevBytAmir/codeln)](https://github.com/DevBytAmir/codeln/releases/latest)
[![License: MIT](https://img.shields.io/github/license/DevBytAmir/codeln)](LICENSE)

A professional source code line counter with advanced filtering.

CodeLn walks a directory tree and reports line statistics: total,
blank, comment, and code lines, broken down per file and per
language, with flexible filtering via CLI flags or a TOML config file.

## Features

- Per-file and per-language line breakdowns (total, blank, comment, code).
- Three output formats: colorized `text`, `json`, and `csv`.
- Filtering by extension, directory, or exact filename, either additively
  (excludes) or as a strict allow-list (`--include-ext`).
- Optional TOML config file, with CLI flags overriding it field-by-field.
- Sensible built-in excludes for common VCS/build/dependency directories
  (`.git`, `target`, `node_modules`, `vendor`, etc.), no setup required.
- Symlink handling and directory-depth limiting via `--follow-links` /
  `--max-depth`.

## Install

Download a prebuilt binary for Linux, macOS, or Windows from the
[latest release](https://github.com/DevBytAmir/codeln/releases/latest), or
build from source:

```sh
cargo install --path .
```

## Usage

```sh
codeln                                   # Analyze current directory
codeln ./src --show-details              # Per-file breakdown
codeln -c codeln.toml                    # Use a config file
codeln -i rs,py -x txt,md                # Filter by extension
codeln -d target,build --max-depth 5     # Exclude directories, limit depth
codeln --format json                     # or: csv, text (default)
```

Run `codeln --init` to generate a `codeln.toml` in the current
directory with all available options documented.

Run `codeln --help` for the full command reference.

## Options

| Flag | Description |
|---|---|
| `PATH` | Directory or file to analyze (default: `.`) |
| `-c, --config <FILE>` | TOML config file; CLI flags override it |
| `-i, --include-ext <EXT,...>` | Only include these extensions |
| `-x, --exclude-ext <EXT,...>` | Exclude these extensions |
| `-d, --exclude-dirs <DIR,...>` | Exclude these directories |
| `-f, --exclude-files <FILE,...>` | Exclude these exact filenames |
| `--no-count-blank` | Exclude blank lines from statistics |
| `--no-count-comments` | Exclude comment lines from statistics |
| `--follow-links` | Follow symbolic links during traversal |
| `--max-depth <N>` | Limit directory recursion depth |
| `--show-details` | Show per-file line counts |
| `--show-excluded` | Show files and directories excluded by filters |
| `--format <text\|json\|csv>` | Output format (overrides the config file's `format`, default: `text`) |
| `--init` | Generate a default `codeln.toml` |

By default, `target`, `node_modules`, `.git`, and other common
build/VCS directories are excluded; the generated `codeln.toml` lists
them all and can be edited to customize.

## Development

```sh
cargo build              # debug build
cargo test                # unit test suite
cargo clippy --all-targets  # lints
cargo fmt                 # formatting
```

See [CHANGELOG.md](CHANGELOG.md) for release history.

## License

[MIT](LICENSE)
