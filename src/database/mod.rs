mod config;
mod data;
mod find;

use std::path::{Path, PathBuf};
pub use find::find_database_from_current_dir;

#[derive(Debug)]
pub struct Database {
    path: PathBuf,
    config: config::DatabaseConfig,
    data: data::DatabaseData,
}

impl Database {

    pub fn save(&self) {
        self.data.save(&self.path);
    }

    pub fn add_tag(&mut self, file: &Path, tag: &String) {
        // TODO Remove unwrap
        let relative_path = find::path_relative_to_db_root(file, &self.root_dir().unwrap())
            .unwrap();
        self.data.add_tag(&relative_path, &tag);
    }


    fn root_dir<'a>(&'a self) -> Option<&'a Path> {
        self.path.parent()
    }

    /// Creates a new database in `path` directory
    pub fn init(path: &Path) -> Option<Self> {
        if !is_valid_init_dir(path) {
            return None;
        }
        let database_dir = path.join(find::DATABASE_DIR);
        std::fs::create_dir(&database_dir).ok()?;
        Some(Self {
            path: std::fs::canonicalize(database_dir).ok()?,
            config: Default::default(),
            data: Default::default(),
        })
    }

    pub fn load(path: PathBuf) -> Option<Self> {
        Some(Self {
            config: config::get_database_config(&path),
            data: data::DatabaseData::load(&path)?,
            path: std::fs::canonicalize(path).ok()?,
        })
    }

    /// Loads the database from the first ancestor with a existing database if any exist
    pub fn load_from_current_dir() -> Option<Self> {
        let path = find_database_from_current_dir()?;
        return Database::load(path);
    }

}

/// Returns true if `path` is valid to be the root of a new database
fn is_valid_init_dir(path: &Path) -> bool {
    path.is_dir() && !find::contains_database_dir(path)
}
