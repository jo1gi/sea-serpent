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
    Add(TaggingArgs),
    /// Remove files that does not exist from database
    Cleanup,
    /// Print information about file
    Info(InfoArgs),
    /// Initialize new database in current directory
    Init,
    /// Remove tag from files
    Remove(TaggingArgs),
    /// Rename files
    Rename(RenameArgs),
    /// Search in database
    Search(SearchArgs),
}

#[derive(StructOpt)]
pub struct TaggingArgs {
    /// Tags to files
    #[structopt(short, long)]
    pub tags: Vec<String>,
    #[structopt(flatten)]
    pub file_selection: FileSelection,
}


/// Cli options for selecting files
#[derive(StructOpt)]
pub struct FileSelection {
    /// Select files recursively through folders
    #[structopt(short, long)]
    pub recursive: bool,
    /// Select directories
    #[structopt(long)]
    pub include_dirs: bool,
    /// Don't select files
    #[structopt(long)]
    pub exclude_files: bool,
    /// Select files from stdin
    #[structopt(long)]
    pub stdin: bool,
    /// List of files
    #[structopt(short, long)]
    pub files: Vec<PathBuf>,
}


// Convert file selection cli arguments to internal system
impl Into<FileSearchSettings> for &FileSelection {
    fn into(self) -> FileSearchSettings {
        FileSearchSettings {
            recursive: self.recursive,
            stdin: self.stdin,
            filetype_filter: match (self.include_dirs, self.exclude_files) {
                (false, false) => FiletypeFilter::FilesOnly,
                (false, true) => FiletypeFilter::Nothing,
                (true, false) => FiletypeFilter::All,
                (true, true) => FiletypeFilter::FoldersOnly,
            },
        }
    }
}

#[derive(StructOpt)]
pub struct InfoArgs {
    #[structopt(flatten)]
    pub file_selection: FileSelection,
}


#[derive(StructOpt)]
pub struct RenameArgs {
    /// Rename template
    #[structopt(long)]
    pub template: String,
    #[structopt(flatten)]
    pub file_selection: FileSelection,
}


#[derive(StructOpt)]
pub struct SearchArgs {
    /// Print results as json
    #[structopt(long)]
    pub json: bool,
    /// Attribute to sort output by
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
    pub limit: Option<usize>,
    /// Search query
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
