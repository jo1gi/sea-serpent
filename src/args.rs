use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Arguments {
    /// Subcommand
    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(StructOpt)]
pub enum Command {
    /// Add tag to files
    Add(AddArgs),
    /// Initialize new database in current directory
    Init,
    /// Search in database
    Search(SearchArgs),
}

#[derive(StructOpt)]
pub struct AddArgs {
    #[structopt(short, long)]
    pub tag: String,
    #[structopt(short, long)]
    pub recursive: bool,
    pub files: Vec<PathBuf>,
}

#[derive(StructOpt)]
pub struct SearchArgs {
    pub search_terms: Vec<String>,
}
