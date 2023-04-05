use thiserror::Error;
use displaydoc::Display;
use std::path::PathBuf;

#[derive(Debug, Error, Display)]
/// Seaserpent database error
pub enum DatabaseError {
    /// Can't retrieve current directory
    CurrentDirNotFound,
    /// Could not find any database from current directory
    DatabaseNotFound,
    /// Can't read {0} from disk
    ReadFromDisk(PathBuf),
    /// Can't write to file: {0}
    WriteToDisk(PathBuf),
    /// Database is not formatted correctly
    DatabaseNotFormattedCorrect,
    /// Can't find root directory of database
    RootDirNotFound,
    /// Can't find file {0}
    FileNotFound(PathBuf),
    /// {0}
    DatabaseConnection(#[from] diesel::ConnectionError),
    /// {0}
    Diesel(#[from] diesel::result::Error),
    /// Invalid database dir
    InvalidRootDir,
    /// Failed to setup database
    DBSetup,
}
