use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use super::DatabaseError;
use crate::search::{SearchExpression, UnaryOp, BinaryOp};

const DATA_FILE: &'static str = "data.json";

type Tag = String;
type File = PathBuf;


#[derive(Default, Debug, Deserialize, Serialize)]
pub struct DatabaseData {
    files: HashMap<File, FileData>,
}

#[derive(serde::Serialize)]
pub struct SearchResult<'a> {
    pub path: &'a PathBuf,
    pub tags: &'a HashSet<String>,
    pub attributes: &'a HashMap<String, Vec<String>>,
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

    /// Remove file from database
    pub fn remove_file(&mut self, file: &Path) {
        self.files.remove(file);
    }

    /// Returns an iterator with all files
    pub fn get_files(&self) -> ReturnFiles {
        self.files.iter()
    }

    /// Get information about file
    pub fn get_file(&self, file: &Path) -> Option<(&File, &FileData)> {
        self.files.get_key_value(file)
    }

    /// Search for files matching `search_term`
    pub fn search(&self, search_term: SearchExpression) -> Vec<SearchResult> {
        self.files.iter()
            .filter(|(_path, filedata)| match_search_query(&filedata, &search_term))
            .map(|(path, filedata)| SearchResult {
                path,
                tags: &filedata.tags,
                attributes: &filedata.attributes
            })
            .collect()
    }

}

fn match_search_query(filedata: &FileData, search_term: &SearchExpression) -> bool {
    match search_term {
        SearchExpression::Tag(tag) => filedata.tags.contains(tag),
        SearchExpression::Attribute { key, value } => filedata.has_attribute(key, value),
        SearchExpression::BinaryOp{ left, right, op_type } => {
            match op_type {
                BinaryOp::And =>
                    match_search_query(filedata, &left) && match_search_query(filedata, &right),
                BinaryOp::Or =>
                    match_search_query(filedata, &left) || match_search_query(filedata, &right),
            }
        },
        SearchExpression::UnaryOp{ expr, op_type } => {
            match op_type {
                UnaryOp::Not => !match_search_query(filedata, &expr)
            }
        },
        SearchExpression::Empty => true
    }
}

type ReturnFiles<'a> = std::collections::hash_map::Iter<'a, PathBuf, FileData>;

/// Store tags and attributes for a file
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct FileData {
    /// Tags on file
    pub tags: HashSet<Tag>,
    /// Key value attributes on file
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

#[cfg(test)]
mod test {

    use std::str::FromStr;

    #[test]
    fn add_tag() {
        let mut data = super::DatabaseData::default();
        let path = std::path::PathBuf::from_str("test_file").unwrap();
        let tag = "test_tag".to_string();
        data.add_tag(&path, &tag);
        assert!(data.files[&path].tags.contains(&tag));
    }

}
