mod args;
mod database;

use args::{Command, AddArgs};
use structopt::StructOpt;

use thiserror::Error;
use displaydoc::Display;

#[derive(Debug, Error, Display)]
/// Seaserpent error
pub enum SeaSerpentError {
    /// {0}
    Database(#[from] database::DatabaseError)
}

fn main() -> Result<(), SeaSerpentError> {
    let args = args::Arguments::from_args();
    match args.command {
        Command::Add(add_args) => add_tags(&add_args),
        Command::Init => initialize_database(),
    }?;
    Ok(())
}

fn add_tags(args: &AddArgs) -> Result<(), SeaSerpentError> {
    let mut database = database::Database::load_from_current_dir().unwrap();
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
