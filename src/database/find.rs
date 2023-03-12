use std::path::{PathBuf, Path};
use super::DatabaseError;
use colored::Colorize;

pub const DATABASE_DIR: &'static str = ".sea-serpent";

/// Returns true if `path` contains a database directory
pub fn contains_database_dir(path: &Path) -> bool {
    for entry in path.read_dir().unwrap() {
        if let Ok(entry) = entry {
            let path: PathBuf = entry.path();
            if entry.file_name() == DATABASE_DIR && path.is_dir() {
                return true;
            }
        }
    }
    return false;
}

/// Returns the first parent of `path` containing a database dir
fn find_database_dir(mut path: &Path) -> Result<PathBuf, DatabaseError> {
    loop {
        if contains_database_dir(path) {
            let database_dir = path.join(DATABASE_DIR);
            return Ok(database_dir);
        }
        if let Some(parent) = path.parent() {
            path = parent;
        } else {
            return Err(DatabaseError::DatabaseNotFound);
        }
    }
}

/// Returns the path of the nearest database directory from the current path
pub fn find_database_from_current_dir() -> Result<PathBuf, DatabaseError> {
    std::env::current_dir()
        .or(Err(DatabaseError::CurrentDirNotFound))
        .and_then(|current_dir| find_database_dir(&current_dir))
}

/// Returns the path of `path` relative to the database root directory
pub fn path_relative_to_db_root(path: &Path, database_root: &Path) -> Result<PathBuf, DatabaseError> {
    log::trace!("Getting relative path of {}", path.to_string_lossy().blue());
    get_full_path(path)?
        .strip_prefix(database_root)
        .or_else(|_| Err(DatabaseError::FileNotFound(path.to_path_buf())))
        .map(|x| x.to_path_buf())
}

/// Get absolute path from relative path
pub fn get_full_path(path: &Path) -> Result<PathBuf, DatabaseError> {
    // TODO Replace with std::path::absolute when it becomes available
    std::fs::canonicalize(path)
        .map_err(|_| DatabaseError::FileNotFound(path.to_path_buf()))
}
