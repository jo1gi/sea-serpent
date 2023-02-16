use std::path::{Path, PathBuf};
use std::collections::{BTreeMap, HashSet};
use serde::{Deserialize, Serialize};

const DATA_FILE: &'static str = "data.json";

type Tag = String;
type File = PathBuf;

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct DatabaseData {
    files: BTreeMap<File, HashSet<Tag>>,
    tags: BTreeMap<Tag, HashSet<File>>,
}

fn create_data_path(database_path: &Path) -> PathBuf {
    database_path.join(DATA_FILE)
}

impl DatabaseData {

    /// Load data file from disk
    pub fn load(database_path: &Path) -> Option<Self> {
        let data_path = create_data_path(database_path);
        let raw_data = std::fs::read_to_string(data_path).ok()?;
        return serde_json::from_str(&raw_data).ok();
    }

    /// Save data file to disk
    pub fn save(&self, database_path: &Path) {
        let data_path = create_data_path(database_path);
        let raw_data = serde_json::to_string(self).unwrap();
        std::fs::write(data_path, raw_data);
    }

    /// Add tag to file
    pub fn add_tag(&mut self, file: &Path, tag: &Tag) {
        if !self.files.contains_key(file) {
            self.files.insert(file.to_path_buf(), HashSet::new());
        }
        if !self.tags.contains_key(tag) {
            self.tags.insert(tag.clone(), HashSet::new());
        }
        self.files.get_mut(file).unwrap().insert(tag.clone());
        self.tags.get_mut(tag).unwrap().insert(file.to_path_buf());
    }

}
