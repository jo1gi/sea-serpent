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
    /// Remove files that does not exist from database
    Cleanup,
    /// Print information about file
    Info(InfoArgs),
    /// Initialize new database in current directory
    Init,
    /// Remove tag from files
    Remove(AddArgs),
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
pub struct InfoArgs {
    pub files: Vec<PathBuf>,
}

#[derive(StructOpt)]
pub struct SearchArgs {
    /// Print results as json
    #[structopt(long)]
    pub json: bool,
    #[structopt(long)]
    pub sort_by: Option<String>,
    /// Print absolute path instead of relative
    #[structopt(long)]
    pub absolute_path: bool,
    /// Print more information about a file
    #[structopt(long)]
    pub info: bool,
    /// Limit the number of results
    #[structopt(long)]
    pub limit: usize,
    pub search_terms: Vec<String>,
}

impl Into<crate::logging::SearchPrintOptions> for &SearchArgs {
    fn into(self) -> crate::logging::SearchPrintOptions {
        crate::logging::SearchPrintOptions {
            json: self.json,
            info: self.info,
        }
    }
}
