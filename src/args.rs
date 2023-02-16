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
}

#[derive(StructOpt)]
pub struct AddArgs {
    #[structopt(short, long)]
    pub tag: String,
    pub files: Vec<PathBuf>,
}
