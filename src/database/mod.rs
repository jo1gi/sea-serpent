mod config;
mod data;
mod error;
mod find;
mod search;

use std::path::{Path, PathBuf};
pub use find::find_database_from_current_dir;
pub use error::DatabaseError;
pub use search::SearchResult;

#[derive(Debug)]
pub struct Database {
    path: PathBuf,
    _config: config::DatabaseConfig,
    data: data::DatabaseData,
}

impl Database {

    /// Save database to disk
    pub fn save(&self) -> Result<(), DatabaseError> {
        self.data.save(&self.path)
    }

    /// Add tag to file
    pub fn add_tag(&mut self, file: &Path, tag: &String) -> Result<(), DatabaseError> {
        // TODO Remove unwrap
        let relative_path = find::path_relative_to_db_root(file, &self.root_dir()?)?;
        self.data.add_tag(&relative_path, &tag);
        log::debug!("Added tag {} to {:?}", tag, relative_path);
        Ok(())
    }


    /// Returns the root directory of the database
    fn root_dir<'a>(&'a self) -> Result<&'a Path, DatabaseError> {
        self.path.parent()
            .ok_or(DatabaseError::RootDirNotFound)
    }

    /// Creates a new database in `path` directory
    pub fn init(path: &Path) -> Option<Self> {
        if !is_valid_init_dir(path) {
            return None;
        }
        let database_dir = path.join(find::DATABASE_DIR);
        std::fs::create_dir(&database_dir).ok()?;
        Some(Self {
            path: find::get_full_path(&database_dir).ok()?,
            _config: Default::default(),
            data: Default::default(),
        })
    }

    /// Load database from disk located in `path`
    pub fn load(path: PathBuf) -> Result<Self, DatabaseError> {
        Ok(Self {
            _config: config::get_database_config(&path),
            data: data::DatabaseData::load(&path)?,
            path: find::get_full_path(&path)?,
        })
    }

    /// Loads the database from the first ancestor with a existing database if any exist
    pub fn load_from_current_dir() -> Result<Self, DatabaseError> {
        let path = find_database_from_current_dir()?;
        return Database::load(path);
    }

}

/// Returns true if `path` is valid to be the root of a new database
fn is_valid_init_dir(path: &Path) -> bool {
    path.is_dir() && !find::contains_database_dir(path)
}
