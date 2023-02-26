mod config;
mod data;
mod error;
mod find;
mod search;

use std::path::{Path, PathBuf};
pub use find::find_database_from_current_dir;
pub use error::DatabaseError;
pub use search::{SearchResult, sort_by_attribute};

#[derive(Debug)]
pub struct Database {
    path: PathBuf,
    config: config::DatabaseConfig,
    data: data::DatabaseData,
}

#[derive(Debug)]
enum Tag {
    /// Basic
    Tag(String),
    /// Attribute with key and value
    Attribute{
        key: String,
        value: String
    }
}

impl Database {

    /// Save database to disk
    pub fn save(&self) -> Result<(), DatabaseError> {
        self.data.save(&self.path)
    }

    /// Add tag to file
    pub fn add_tag(&mut self, file: &Path, tag: &String) -> Result<(), DatabaseError> {
        let relative_path = find::path_relative_to_db_root(file, &self.root_dir()?)?;
        self.config.get_alias(tag)
            .unwrap_or_else(|| vec![tag])
            .iter()
            .map(|tag| parse_tag(tag))
            .filter(|tag| match tag {
                Tag::Tag(x) => self.config.tag_allowed(x),
                Tag::Attribute{key, value: _} => self.config.tag_allowed(key),
            })
            .for_each(|tag| {
                log::debug!("Adding tag {:?} to {}", tag, file.display());
                match tag {
                    Tag::Tag(tag) => self.data.add_tag(&relative_path, &tag),
                    Tag::Attribute{key, value} => self.data.add_attribute(&relative_path, key, value)
                }
            });
        Ok(())
    }

    /// Remove tag from file
    pub fn remove_tag(&mut self, file: &Path, tag: &String) -> Result<(), DatabaseError> {
        let relative_path = find::path_relative_to_db_root(file, &self.root_dir()?)?;
        self.config.get_alias(tag)
            .unwrap_or_else(|| vec![tag])
            .iter()
            .for_each(|tag| {
                log::debug!("Removing tag {} from {}", tag, file.display());
                self.data.remove_tag(&relative_path, &tag);
            });
        Ok(())
    }

    /// Returns the root directory of the database
    fn root_dir(&self) -> Result<&Path, DatabaseError> {
        self.path.parent()
            .ok_or(DatabaseError::RootDirNotFound)
    }

    /// Returns absolute path of a file in the database
    fn get_absolute_path(&self, file_path: &Path) -> Result<PathBuf, DatabaseError> {
        Ok(self.root_dir()?.join(file_path))
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
            config: Default::default(),
            data: Default::default(),
        })
    }

    /// Load database from disk located in `path`
    pub fn load(path: PathBuf) -> Result<Self, DatabaseError> {
        Ok(Self {
            config: config::get_database_config(&path),
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

fn parse_tag(tag: &String) -> Tag {
    match get_attribute(&tag) {
        None => Tag::Tag(tag.to_string()),
        Some((key, value)) => Tag::Attribute{ key, value }
    }
}

fn get_attribute(tag: &str) -> Option<(String, String)> {
    if !tag.contains(":") {
        return None;
    }
    let mut iter = tag.split(":");
    let key = iter.next();
    let value = iter.collect();
    return Some((key.unwrap().to_string(), value));
}
