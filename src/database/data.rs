use std::path::{Path, PathBuf};
use std::collections::{BTreeMap, HashSet};
use serde::{Deserialize, Serialize};
use super::{Database, DatabaseError};

const DATA_FILE: &'static str = "data.json";

type Tag = String;
type File = PathBuf;

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct DatabaseData {
    files: BTreeMap<File, HashSet<Tag>>,
}

fn create_data_path(database_path: &Path) -> PathBuf {
    database_path.join(DATA_FILE)
}

impl DatabaseData {

    /// Load data file from disk
    pub fn load(database_path: &Path) -> Result<Self, DatabaseError> {
        let data_path = create_data_path(database_path);
        let raw_data = std::fs::read_to_string(&data_path)
            .or_else(move |_| Err(DatabaseError::ReadFromDisk(data_path)))?;
        return serde_json::from_str(&raw_data)
            .or(Err(DatabaseError::DatabaseNotFormattedCorrect));
    }

    /// Save data file to disk
    pub fn save(&self, database_path: &Path) -> Result<(), DatabaseError> {
        let data_path = create_data_path(database_path);
        let raw_data = serde_json::to_string(self).unwrap();
        std::fs::write(&data_path, raw_data)
            .or_else(move |_| Err(DatabaseError::WriteToDisk(data_path)))?;
        Ok(())
    }

    /// Add tag to file
    pub fn add_tag(&mut self, file: &Path, tag: &Tag) {
        if !self.files.contains_key(file) {
            self.files.insert(file.to_path_buf(), HashSet::new());
        }
        self.files.get_mut(file).unwrap().insert(tag.clone());
    }

    /// Remove tag from file
    pub fn remove_tag(&mut self, file: &Path, tag: &Tag) {
        if let Some(file_tags) = self.files.get_mut(file) {
            file_tags.remove(tag);
        }
    }

}

type ReturnFiles<'a> = std::collections::btree_map::Iter<'a, PathBuf, HashSet<String>>;

impl Database {

    pub fn get_files<'a>(&'a self) -> ReturnFiles {
        self.data.files.iter()
    }

}
