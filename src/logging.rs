use log::{Level, LevelFilter};
use colored::Colorize;

use crate::database::SearchResult;

use thiserror::Error;
use displaydoc::Display;

#[derive(Debug, Error, Display)]
pub enum SeaSerpentLoggingError {
    /// {0}
    Init(#[from] log::SetLoggerError),
    /// {0}
    Json(#[from] serde_json::Error)
}

/// Setup logging system
pub fn setup_logger(level: LevelFilter) -> Result<(), SeaSerpentLoggingError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            match create_prefix(record.level(), record.target()) {
                Some(prefix) => out.finish(format_args!("{} {}", prefix, message)),
                None => out.finish(format_args!("{}", message)),
            }
        })
        .level(level)
        .chain(std::io::stderr())
        .apply()?;
    Ok(())
}

fn create_prefix(level: Level, target: &str) -> Option<String> {
    match level {
        Level::Error => Some("ERROR".red().bold().to_string()),
        Level::Debug => Some(format!("{} {}", "Debug".bold().yellow(), target.bright_black())),
        Level::Trace => Some(format!("{} {}", "Trace".bold().cyan(), target.bright_black())),
        _ => None,
    }
}

pub struct SearchPrintOptions {
    pub json: bool
}

pub fn print_search_result(results: &Vec<SearchResult>, options: SearchPrintOptions) -> Result<(), SeaSerpentLoggingError> {
    if options.json {
        println!("{}", serde_json::to_string_pretty(results)?);
    } else {
        print_search_result_simple(results);
    }
    Ok(())
}

fn print_search_result_simple(results: &Vec<SearchResult>) {
    for result in results {
        println!("{}", result.path.display());
    }
}
