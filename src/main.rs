mod args;
mod database;
mod logging;
mod search;
mod utils;

use args::{Command, AddArgs, SearchArgs};
use structopt::StructOpt;

use thiserror::Error;
use displaydoc::Display;

#[derive(Debug, Error, Display)]
/// Seaserpent error
pub enum SeaSerpentError {
    /// {0}
    Database(#[from] database::DatabaseError),
    /// {0}
    Search(#[from] search::SearchError),
    /// {0}
    Logging(#[from] logging::SeaSerpentLoggingError),
}

fn main() -> Result<(), SeaSerpentError> {
    let args = args::Arguments::from_args();
    logging::setup_logger(args.log_level)?;
    match args.command {
        Command::Add(add_args) => add_tags(&add_args),
        Command::Init => initialize_database(),
        Command::Remove(remove_args) => remove_tags(&remove_args),
        Command::Search(search_args) => search(&search_args),
    }?;
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
    // TODO: Find better way to remove all
    if args.files.is_empty() {
        database.remove_tag_from_all(&args.tag)
    } else {
        for file in utils::files::get_files(&args.files, args.into()) {
            database.remove_tag(&file, &args.tag)?;
        }
    }
    database.save()?;
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

/// Search for files in database
fn search(args: &SearchArgs) -> Result<(), SeaSerpentError> {
    let database = database::Database::load_from_current_dir()?;
    let joined = args.search_terms.join(" ");
    let search_expr = search::parse(&joined)?;
    log::debug!("Search: {:#?}", search_expr);
    let results = database.search(search_expr);
    logging::print_search_result(&results, args.into())?;
    Ok(())
}
