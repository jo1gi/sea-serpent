mod args;
mod database;
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
}

fn main() -> Result<(), SeaSerpentError> {
    let args = args::Arguments::from_args();
    match args.command {
        Command::Add(add_args) => add_tags(&add_args),
        Command::Init => initialize_database(),
        Command::Search(search_args) => search(&search_args),
    }?;
    Ok(())
}

fn add_tags(args: &AddArgs) -> Result<(), SeaSerpentError> {
    let mut database = database::Database::load_from_current_dir()?;
    for file in &args.files {
        database.add_tag(&file, &args.tag)?;
    }
    println!("{:#?}", database);
    database.save()?;
    Ok(())
}

fn initialize_database() -> Result<(), SeaSerpentError> {
    let current_dir = std::env::current_dir().unwrap();
    let db = database::Database::init(&current_dir).unwrap();
    db.save()?;
    Ok(())
}

fn search(args: &SearchArgs) -> Result<(), SeaSerpentError> {
    let database = database::Database::load_from_current_dir()?;
    let joined = args.search_terms.join(" ");
    let search_expr = search::parse(&joined)?;
    println!("Search: {:#?}", search_expr);
    let results = database.search(search_expr);
    println!("Results: {:#?}", results);
    Ok(())
}
