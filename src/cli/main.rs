mod args;
mod logging;

use args::{Command, AddArgs, InfoArgs, RenameArgs, SearchArgs};
use structopt::StructOpt;
use std::str::FromStr;
use seaserpent::{database, format, search, utils};

use thiserror::Error;
use displaydoc::Display;

#[derive(Debug, Error, Display)]
/// Seaserpent error
pub enum SeaSerpentError {
    /// {0}
    Database(#[from] database::DatabaseError),
    /// Failed to format string
    Formatting,
    /// {0}
    Search(#[from] search::SearchError),
    /// {0}
    Logging(#[from] logging::SeaSerpentLoggingError),
}

fn main() -> Result<(), SeaSerpentError> {
    let args = args::Arguments::from_args();
    logging::setup_logger(args.log_level)?;
    let result = match args.command {
        Command::Add(add_args) => add_tags(&add_args),
        Command::Cleanup => cleanup(),
        Command::Info(info_args) => print_info(&info_args),
        Command::Init => initialize_database(),
        Command::Remove(remove_args) => remove_tags(&remove_args),
        Command::Rename(rename_args) => rename(&rename_args),
        Command::Search(search_args) => search(&search_args),
    };
    match result {
        Ok(_) => (),
        Err(error) => log::error!("{}", error),
    };
    Ok(())
}

/// Add new tags to files
fn add_tags(args: &AddArgs) -> Result<(), SeaSerpentError> {
    let mut database = database::Database::load_from_current_dir()?;
    for file in utils::files::get_files(&args.files, args.into()) {
        database.add_tag(&file, &args.tag)?;
    }
    database.save()?;
    Ok(())
}

/// Remove tags from files
fn remove_tags(args: &AddArgs) -> Result<(), SeaSerpentError> {
    let mut database = database::Database::load_from_current_dir()?;
    for file in utils::files::get_files(&args.files, args.into()) {
        database.remove_tag(&file, &args.tag)?;
    }
    database.save()?;
    Ok(())
}

/// Remove files from database that does not exist
fn cleanup() -> Result<(), SeaSerpentError> {
    let mut database = database::Database::load_from_current_dir()?;
    database.cleanup();
    database.save()?;
    Ok(())
}

/// Print info about files
fn print_info(args: &InfoArgs) -> Result<(), SeaSerpentError> {
    let database = database::Database::load_from_current_dir()?;
    for file in &args.files {
        let file_info = database.get_file_info(file)?;
        logging::print_result_descriptive(&file_info);
    }
    Ok(())
}

/// Create new database in current dir
fn initialize_database() -> Result<(), SeaSerpentError> {
    let current_dir = std::env::current_dir().unwrap();
    let db = database::Database::init(&current_dir).unwrap();
    db.save()?;
    log::info!("Created new database");
    Ok(())
}

/// Rename file based on template and attributes
fn rename(rename_args: &RenameArgs) -> Result<(), SeaSerpentError> {
    let mut database = database::Database::load_from_current_dir()?;
    for file in &rename_args.files {
        let fileinfo = database.get_file_info(&file)?;
        let new_path_str = format::format_result(&fileinfo, &rename_args.template)
            .map_err(|_| SeaSerpentError::Formatting)?;
        let new_path = std::path::PathBuf::from_str(&new_path_str).unwrap();
        database.move_file(&file, &new_path)?;
        log::info!("Moved {} to {}", file.display(), new_path.display());
    }
    database.save()?;
    Ok(())
}

/// Search for files in database
fn search(args: &SearchArgs) -> Result<(), SeaSerpentError> {
    let database = database::Database::load_from_current_dir()?;
    let joined = args.search_terms.join(" ");
    let search_expr = search::parse(&joined)?;
    let mut results = database.search(search_expr);
    if let Some(search_by_key) = &args.sort_by {
        database::sort_by_attribute(&mut results, &search_by_key);
    }
    if let Some(limit) = &args.limit {
        results.truncate(*limit);
    }
    logging::print_search_results(&results, args.into())?;
    Ok(())
}
