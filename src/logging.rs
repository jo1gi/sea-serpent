use log::{Level, LevelFilter};
use colored::Colorize;

/// Setup logging system
pub fn setup_logger(level: LevelFilter) -> Result<(), fern::InitError> {
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
