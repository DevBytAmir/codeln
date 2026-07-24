//! Renders a `CountResults` in the format requested via `--format`
//! (text, json, or csv).

use crate::config::Config;
use crate::counter::CountResults;
use anyhow::{Ok, Result};
use colored::Colorize;

/// Dispatches to the renderer for `config.format`.
pub fn print_results(results: &CountResults, config: &Config) -> Result<()> {
    match config.format.as_str() {
        "json" => print_json(results)?,
        "csv" => print_csv(results, config)?,
        _ => print_text(results, config)?,
    }
    Ok(())
}

/// Human-readable, colorized report: per-file details (if enabled), language
/// breakdown, and summary totals.
// A flat sequential report renderer; splitting it up would hurt readability
// more than a line-count metric is worth.
#[allow(clippy::too_many_lines)]
fn print_text(results: &CountResults, config: &Config) -> Result<()> {
    println!(
        "\n{}",
        "╔════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "║        CodeLn - Line Counter Results           ║"
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "╚════════════════════════════════════════════════╝".cyan()
    );

    if config.show_details && !results.file_counts.is_empty() {
        println!("\n{}", "Files Analyzed:".bold().underline());
        println!("{}", "─".repeat(70).dimmed());

        let mut files: Vec<_> = results.file_counts.iter().collect();
        files.sort_by_key(|(_, count)| std::cmp::Reverse(count.total));

        for (file, count) in files {
            println!(
                "  {} │ {} {} {} {}",
                format!("{:>8}", count.total).yellow().bold(),
                format!("Code: {:>6}", count.code).green(),
                format!("Blank: {:>5}", count.blank).blue(),
                format!("Comments: {:>5}", count.comments).magenta(),
                file.dimmed()
            );
        }
        println!();
    }

    if !results.language_stats.is_empty() {
        println!("\n{}", "Language Breakdown:".bold().underline());
        println!("{}", "─".repeat(70).dimmed());

        let mut langs: Vec<_> = results.language_stats.iter().collect();
        langs.sort_by_key(|(_, stats)| std::cmp::Reverse(stats.code_lines));

        for (lang, stats) in langs {
            // `percentage` is always in 0.0..=100.0, so this cast never
            // truncates a meaningful fraction or flips sign.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let bar_length = (stats.percentage / 2.0) as usize;
            let bar = "█".repeat(bar_length.min(40));

            println!(
                "  {} {:>6.1}% │ {} files │ {} lines │ {}",
                format!("{lang:<15}").cyan().bold(),
                stats.percentage,
                format!("{:>4}", stats.files).yellow(),
                format!("{:>6}", stats.code_lines).green(),
                bar.bright_green()
            );
        }
        println!();
    }

    println!("{}", "Summary Statistics:".bold().underline());
    println!("{}", "─".repeat(70).dimmed());
    println!(
        "  {} {}",
        "Total Files:      ".bold(),
        format!("{:>10}", results.total_files).cyan()
    );
    println!(
        "  {} {}",
        "Total Lines:      ".bold(),
        format!("{:>10}", results.total_lines).green().bold()
    );

    if config.count_blank {
        println!(
            "  {} {}",
            "  ├─ Blank Lines: ".dimmed(),
            format!("{:>10}", results.blank_lines).blue()
        );
    }

    if config.count_comments {
        println!(
            "  {} {}",
            "  ├─ Comments:    ".dimmed(),
            format!("{:>10}", results.comment_lines).magenta()
        );
    }

    println!(
        "  {} {}",
        "  └─ Code Lines:  ".dimmed(),
        format!("{:>10}", results.code_lines).green().bold()
    );

    if config.count_blank && config.count_comments {
        // Line counts are nowhere near f64's 2^52 exact-integer limit, so
        // the precision loss clippy warns about here is not a real concern.
        #[allow(clippy::cast_precision_loss)]
        let code_percentage = if results.total_lines > 0 {
            (results.code_lines as f64 / results.total_lines as f64) * 100.0
        } else {
            0.0
        };
        println!(
            "\n  {} {}",
            "Code Percentage:  ".bold(),
            format!("{code_percentage:>9.1}%").yellow()
        );
    }

    if !results.excluded_files.is_empty() && config.show_excluded {
        println!("\n{}", "Excluded Files:".bold().red());
        println!("{}", "─".repeat(70).dimmed());
        for file in &results.excluded_files {
            println!("  {}", file.dimmed());
        }
    }

    println!();
    Ok(())
}

/// Pretty-printed JSON dump of the full `CountResults`.
fn print_json(results: &CountResults) -> Result<()> {
    let json = serde_json::to_string_pretty(results)?;
    println!("{json}");
    Ok(())
}

/// CSV with one row per file (if `show_details`) plus a trailing TOTAL row.
fn print_csv(results: &CountResults, config: &Config) -> Result<()> {
    let mut wtr = csv::Writer::from_writer(std::io::stdout());
    write_csv(&mut wtr, results, config)?;
    wtr.flush()?;
    Ok(())
}

fn write_csv<W: std::io::Write>(
    wtr: &mut csv::Writer<W>,
    results: &CountResults,
    config: &Config,
) -> Result<()> {
    wtr.write_record(["file", "total", "blank", "comments", "code"])?;

    if config.show_details {
        for (file, count) in &results.file_counts {
            wtr.write_record([
                file,
                &count.total.to_string(),
                &count.blank.to_string(),
                &count.comments.to_string(),
                &count.code.to_string(),
            ])?;
        }
    }

    wtr.write_record([
        "TOTAL",
        &results.total_lines.to_string(),
        &results.blank_lines.to_string(),
        &results.comment_lines.to_string(),
        &results.code_lines.to_string(),
    ])?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::counter::FileCount;

    fn sample_results() -> CountResults {
        let mut results = CountResults {
            total_files: 1,
            total_lines: 4,
            blank_lines: 1,
            comment_lines: 1,
            code_lines: 2,
            ..CountResults::default()
        };
        results.file_counts.insert(
            "a.rs".to_string(),
            FileCount {
                total: 4,
                blank: 1,
                comments: 1,
                code: 2,
            },
        );
        results
    }

    #[test]
    fn csv_includes_file_rows_when_show_details_is_set() {
        let results = sample_results();
        let config = Config {
            show_details: true,
            ..Config::default()
        };

        let mut wtr = csv::Writer::from_writer(vec![]);
        write_csv(&mut wtr, &results, &config).unwrap();
        let output = String::from_utf8(wtr.into_inner().unwrap()).unwrap();

        assert!(output.contains("a.rs,4,1,1,2"));
        assert!(output.contains("TOTAL,4,1,1,2"));
    }

    #[test]
    fn csv_omits_file_rows_when_show_details_is_unset() {
        let results = sample_results();
        let config = Config::default();

        let mut wtr = csv::Writer::from_writer(vec![]);
        write_csv(&mut wtr, &results, &config).unwrap();
        let output = String::from_utf8(wtr.into_inner().unwrap()).unwrap();

        assert!(!output.contains("a.rs"));
        assert!(output.contains("TOTAL,4,1,1,2"));
    }

    #[test]
    fn print_results_dispatches_on_config_format() {
        let results = sample_results();
        let mut config = Config {
            format: "json".to_string(),
            ..Config::default()
        };
        assert!(print_results(&results, &config).is_ok());

        config.format = "csv".to_string();
        assert!(print_results(&results, &config).is_ok());

        config.format = "text".to_string();
        assert!(print_results(&results, &config).is_ok());
    }
}
