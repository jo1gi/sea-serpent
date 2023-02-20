use std::path::PathBuf;
use structopt::StructOpt;
use crate::utils::files::{FileSearchSettings, FiletypeFilter};

#[derive(StructOpt)]
pub struct Arguments {
    /// Logging level
    #[structopt(short, long, default_value="info", global = true)]
    pub log_level: log::LevelFilter,
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
    #[structopt(long)]
    pub exclude_dirs: bool,
    #[structopt(long)]
    pub exclude_files: bool,
    pub files: Vec<PathBuf>,
}

impl Into<FileSearchSettings> for &AddArgs {
    fn into(self) -> FileSearchSettings {
        FileSearchSettings {
            recursive: self.recursive,
            filetype_filter: if self.exclude_dirs {
                FiletypeFilter::FilesOnly
            } else if self.exclude_files {
                FiletypeFilter::FoldersOnly
            }else {
                FiletypeFilter::All
            },
        }
    }
}

#[derive(StructOpt)]
pub struct SearchArgs {
    pub search_terms: Vec<String>,
}
