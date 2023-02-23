use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use super::DatabaseError;

const DATA_FILE: &'static str = "data.json";

type Tag = String;
type File = PathBuf;

/// Store tags and attributes for a file
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct FileData {
    pub tags: HashSet<Tag>,
    pub attributes: HashMap<String, Vec<String>>,
}

impl FileData {

    pub fn has_attribute(&self, key: &Option<String>, value: &Option<String>) -> bool {
        match (key, value) {
            (Some(key), None) => self.attributes.contains_key(key),
            (Some(key), Some(value)) =>
                self.attributes.get(key)
                    .map(|values| values.contains(value))
                    .unwrap_or(false),
            (None, Some(value)) =>
                self.attributes
                    .values()
                    .flat_map(|values| values)
                    .find(|x| value == *x)
                    .is_some(),
            (None, None) => true,
        }
    }

}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct DatabaseData {
    files: HashMap<File, FileData>,
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

    /// Make sure data exists for `file`
    fn prepare_file(&mut self, file: &Path) {
        if !self.files.contains_key(file) {
            self.files.insert(file.to_path_buf(), FileData::default());
        }
    }

    /// Add tag to file
    pub fn add_tag(&mut self, file: &Path, tag: &Tag) {
        self.prepare_file(file);
        self.files.get_mut(file).unwrap().tags.insert(tag.clone());
    }

    /// Add attribute to file
    pub fn add_attribute(&mut self, file: &Path, key: String, value: String) {
        self.prepare_file(file);
        let filedata = self.files.get_mut(file).unwrap();
        if !filedata.attributes.contains_key(&key) {
            filedata.attributes.insert(key.clone(), Vec::new());
        }
        filedata.attributes.get_mut(&key).unwrap().push(value);
    }

    /// Remove tag from file
    pub fn remove_tag(&mut self, file: &Path, tag: &Tag) {
        if let Some(filedata) = self.files.get_mut(file) {
            filedata.tags.remove(tag);
        }
    }

    /// Returns an iterator with all files
    pub fn get_files(&self) -> ReturnFiles {
        self.files.iter()
    }

}

type ReturnFiles<'a> = std::collections::hash_map::Iter<'a, PathBuf, FileData>;
