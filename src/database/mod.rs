mod config;
mod storage;
mod error;
mod find;
mod tag;

use std::{path::{Path, PathBuf}, cmp::Ordering};
use colored::Colorize;
pub use find::find_database_from_current_dir;
pub use error::DatabaseError;
pub use storage::SearchResult;
pub use tag::Tag;

/// Seaserpent database
pub struct Database {
    path: PathBuf,
    config: config::DatabaseConfig,
    storage: storage::DatabaseStorage,
}


impl Database {

    /// Add tag to file
    pub fn add_tag(&mut self, file: &Path, tag: &String) -> Result<(), DatabaseError> {
        let relative_path = find::path_relative_to_db_root(file, &self.root_dir()?)?;
        let tags = self.config.get_alias(tag)
            .unwrap_or_else(|| vec![tag]);
        // Filter tags
        let iter = tags
            .iter()
            .map(|tag| Tag::new(tag))
            .filter(|tag| self.config.tag_allowed(tag));
        for tag in iter {
            log::debug!("Adding tag, {}, to {}", tag.to_string(), file.display());
            // Add to storage
            let result = match tag {
                Tag::Key(tag) => self.storage.add_tag(&relative_path, &tag),
                Tag::KeyValue{key, value} => self.storage.add_attribute(&relative_path, key, value)
            };
            // Handle error
            match result {
                Ok(_) => (),
                err => return err,
            }

        }
        Ok(())
    }

    /// Remove tag from file
    pub fn remove_tag(&mut self, file: &Path, tag: &String) -> Result<(), DatabaseError> {
        let relative_path = find::path_relative_to_db_root(file, &self.root_dir()?)?;
        let alias = self.config.get_alias(tag);
        let tags = alias.unwrap_or_else(|| vec![tag]);
        for unparsed_tag in tags {
            let parsed_tag = Tag::new(unparsed_tag);
            log::debug!(
                "Removing tag {:?} from {}",
                parsed_tag.to_string(),
                file.to_string_lossy().blue()
            );
            self.storage.remove_tag(&relative_path, &parsed_tag)?;
        }
        Ok(())
    }

    /// Returns the root directory of the database
    fn root_dir(&self) -> Result<&Path, DatabaseError> {
        self.path.parent()
            .ok_or(DatabaseError::RootDirNotFound)
    }

    /// Returns absolute path of a file in the database
    pub fn get_absolute_path(&self, file_path: &Path) -> Result<PathBuf, DatabaseError> {
        Ok(self.root_dir()?.join(file_path))
    }

    /// Creates a new database in `path` directory
    pub fn init(path: &Path) -> Result<Self, DatabaseError> {
        if !is_valid_init_dir(path) {
            return Err(DatabaseError::InvalidRootDir);
        }
        let database_dir = path.join(find::DATABASE_DIR);
        std::fs::create_dir(&database_dir)
            .map_err(|_| DatabaseError::WriteToDisk(database_dir.clone()))?;
        Ok(Self {
            path: find::get_full_path(&database_dir)?,
            config: Default::default(),
            storage: storage::DatabaseStorage::load(&database_dir)?,
        })
    }

    /// Load database from disk located in `path`
    pub fn load(path: PathBuf) -> Result<Self, DatabaseError> {
        log::debug!("Loading database from {}", path.to_string_lossy().blue());
        Ok(Self {
            config: config::get_database_config(&path),
            storage: storage::DatabaseStorage::load(&path)?,
            path: find::get_full_path(&path)?,
        })
    }

    /// Loads the database from the first ancestor with a existing database if any exist
    pub fn load_from_current_dir() -> Result<Self, DatabaseError> {
        let path = find_database_from_current_dir()?;
        return Database::load(path);
    }

    /// Clean up the database in multiple ways:
    /// - Remove files from database that does not exist anymore
    /// - Remove tags from that are not allowed (by whitelist or blacklist)
    pub fn cleanup(&mut self) -> Result<(), DatabaseError> {
        // Remove files that does not exist
        let files_to_remove: Vec<PathBuf> = self.storage.get_all_files()?
            .into_iter()
            .filter(|result| {
                self.get_absolute_path(&result.path)
                    .ok()
                    .map(|path| !path.exists())
                    .unwrap_or(false)
            })
            .map(|result| result.path)
            .collect();
        for file in files_to_remove {
            log::debug!("Removing {} from database", file.to_string_lossy().blue());
            self.storage.remove_file(&file)?;
        }
        // Remove tags that are not allowed
        let unallowed_tags = self.storage.get_all_files()?
            .into_iter()
            .map(|result| {
                let unallowed_tags = result.tags
                    .iter()
                    .map(|tag| Tag::new(tag))
                    .filter(|tag| !self.config.tag_allowed(tag))
                    .map(|tag| tag)
                    .collect::<Vec<_>>();
                (result.path, unallowed_tags)
            })
            .collect::<Vec<_>>();
        for (path, tags) in unallowed_tags {
            for tag in tags {
                log::debug!("Removing {} from {}", tag.to_string(), path.to_string_lossy());
                self.storage.remove_tag(&path, &tag)?;
            }
        }
        Ok(())
    }

    /// Search for files matching `search_term`
    pub fn search(&mut self, search_term: crate::search::SearchExpression) -> Result<Vec<SearchResult>, DatabaseError> {
        let mut results = self.storage.search(search_term)?;
        results.sort_by(sort_by_path);
        Ok(results)
    }

    /// Move all data about `original_path` to `new_path`,
    /// both in the database and on the filesystem
    pub fn move_file(&mut self, original: &Path, new: &Path) -> Result<(), DatabaseError> {
        let root_dir = self.root_dir()?;
        let original_relative = find::path_relative_to_db_root(original, &root_dir)?;
        // The file is moved here because `path_relative_to_db_root` requires the file to exist
        // This can be fixed if. std::path::absolute comes out of nightly
        std::fs::rename(original, new)
            .map_err(|_| DatabaseError::WriteToDisk(new.to_path_buf()))?;
        let new_relative = find::path_relative_to_db_root(new, &root_dir)?;
        self.storage.move_file(&original_relative,new_relative)
    }

    /// Return file tags and attributes
    pub fn get_file_info(&mut self, file: &Path) -> Result<SearchResult, DatabaseError> {
        let relative_path = find::path_relative_to_db_root(file, &self.root_dir()?)?;
        self.storage.get_file_from_path(&relative_path)
            .map_err(|_| DatabaseError::FileNotFound(file.to_path_buf()))
    }

}

fn get_first_attribute<'a>(result: &'a SearchResult, key: &str) -> Option<&'a String> {
    result.attributes.iter()
        .find(|(x, _)| key == x)
        .map(|(_, value)| value)
}

fn sort_by_path(a: &SearchResult, b: &SearchResult) -> Ordering {
    a.path.cmp(&b.path)
}

pub fn sort_by_attribute(results: &mut Vec<SearchResult>, key: &str) {
    results.sort_by(|a, b| {
        let a_value = get_first_attribute(a, key);
        let b_value = get_first_attribute(b, key);
        a_value.cmp(&b_value)
    })
}

/// Returns true if `path` is valid to be the root of a new database
fn is_valid_init_dir(path: &Path) -> bool {
    path.is_dir() && !find::contains_database_dir(path)
}
