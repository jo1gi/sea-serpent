use std::path::{PathBuf, Path};

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
fn find_database_dir(mut path: &Path) -> Option<PathBuf> {
    loop {
        if contains_database_dir(path) {
            let database_dir = path.join(DATABASE_DIR);
            return Some(database_dir);
        }
        if let Some(parent) = path.parent() {
            path = parent;
        } else {
            return None;
        }
    }
}

/// Returns the path of the nearest database directory from the current path
pub fn find_database_from_current_dir() -> Option<PathBuf> {
    if let Ok(current_dir) = std::env::current_dir() {
        find_database_dir(&current_dir)
    } else {
        None
    }
}

/// Returns the path of `path` relative to the database root directory
pub fn path_relative_to_db_root(path: &Path, database_root: &Path) -> Option<PathBuf> {
    std::fs::canonicalize(path).ok()?
        .strip_prefix(database_root).ok()
        .map(|x| x.to_path_buf())
}
