mod config;
mod data;
mod error;
mod find;

use std::path::{Path, PathBuf};
use colored::Colorize;
pub use find::find_database_from_current_dir;
pub use error::DatabaseError;
pub use data::SearchResult;

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
                Tag::Attribute{key, value: _} => self.config.tag_allowed(&format!("{}:", key)),
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
            .map(|tag| parse_tag(tag))
            .for_each(|tag| {
                log::debug!("Removing tag {:?} from {}", tag, file.to_string_lossy().blue());
                match tag {
                    Tag::Tag(tag) => self.data.remove_tag(&relative_path, &tag),
                    Tag::Attribute{key, value} => self.data.remove_attribute(&relative_path, key, value),
                }
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
        log::debug!("Loading database from {}", path.to_string_lossy().blue());
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

    /// Clean up the database in multiple ways:
    /// - Remove files from database that does not exist anymore
    /// - Remove tags from that are not allowed (by whitelist or blacklist)
    pub fn cleanup(&mut self) {
        // Remove files that does not exist
        let files_to_remove: Vec<PathBuf> = self.data.get_all_files()
            .map(|(path, _filedata)| path.clone())
            .filter(|path| !path.exists())
            .collect();
        for file in files_to_remove {
            log::debug!("Removing {} from database", file.to_string_lossy().blue());
            self.data.remove_file(&file);
        }
        // Remove tags that are not allowed
        let unallowed_tags = self.data.get_all_files()
            .map(|(path, filedata)| {
                let unallowed_tags = filedata.tags
                    .iter()
                    .filter(|tag| !self.config.tag_allowed(tag))
                    .map(|tag| tag.clone())
                    .collect::<Vec<_>>();
                (path.clone(), unallowed_tags)
            })
            .collect::<Vec<_>>();
        for (path, tags) in unallowed_tags {
            for tag in tags {
                log::debug!("Removing {tag} from {}", path.to_string_lossy());
                self.data.remove_tag(&path, &tag);
            }
        }
    }

    /// Search for files matching `search_term`
    pub fn search(&self, search_term: crate::search::SearchExpression) -> Vec<SearchResult> {
        let mut results = self.data.search(search_term);
        results.sort_by_key(|result| result.path);
        return results;
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
        self.data.move_file(&original_relative,new_relative)
    }

    /// Return file tags and attributes
    pub fn get_file_info(&self, file: &Path) -> Result<SearchResult, DatabaseError> {
        let relative_path = find::path_relative_to_db_root(file, &self.root_dir()?)?;
        self.data.get_file(&relative_path)
            .map(|(path, filedata)| SearchResult {
                path,
                tags: &filedata.tags,
                attributes: &filedata.attributes,
            })
            .ok_or_else(|| DatabaseError::FileNotFound(file.to_path_buf()))
    }

}

fn get_first_attribute<'a>(result: &'a SearchResult, key: &str) -> Option<&'a String> {
    result.attributes.get(key)
        .and_then(|values| {
            let mut new_vec: Vec<_> = values.iter().collect();
            new_vec.sort();
            new_vec.first().map(|x| *x)
        })
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
