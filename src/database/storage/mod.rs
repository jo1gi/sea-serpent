mod models;

use super::{DatabaseError, Tag};
use crate::search::{SearchExpression, UnaryOp, BinaryOp};

use std::{
    str::FromStr,
    path::{Path, PathBuf},
    collections::HashSet,
};
use diesel::{
    self,
    prelude::*,
    sqlite::SqliteConnection,
    result::Error as DieselError,
    result::DatabaseErrorKind,
    Connection, RunQueryDsl, QueryDsl,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness,};


/// Name of sqlite file
const DATA_FILE: &'static str = "data.sqlite";

/// Sql migration data
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");


pub struct DatabaseStorage {
    /// Connection to sqlite database
    connection: SqliteConnection,
}


#[derive(serde::Serialize)]
pub struct SearchResult {
    pub path: PathBuf,
    pub tags: HashSet<String>,
    pub attributes: Vec<(String, String)>
}


/// Create full path to sqlite file
fn create_data_path(database_path: &Path) -> PathBuf {
    database_path.join(DATA_FILE)
}

impl DatabaseStorage {


    /// Load data file from disk
    pub fn load(database_path: &Path) -> Result<Self, DatabaseError> {
        let data_path = create_data_path(database_path);
        let data_str = data_path.to_str()
            .ok_or(DatabaseError::DatabaseNotFound)?;
        let mut connection = SqliteConnection::establish(&data_str)?;
        connection.run_pending_migrations(MIGRATIONS)
            .map_err(|_| DatabaseError::DBSetup)?;
        let data = Self { connection };
        return Ok(data);
    }


    fn get_file_id(&mut self, file: &Path) -> Result<i32, DatabaseError> {
        let path_str = file.to_string_lossy().to_string();
        let file_ids = models::files::table
            .filter(models::files::path.is(&path_str))
            .limit(1)
            .select(models::files::id)
            .load::<i32>(&mut self.connection)?;
        if file_ids.len() > 0 {
            Ok(file_ids[0])
        } else {
            Err(DatabaseError::FileNotFound(file.to_path_buf()))
        }
    }


    /// Create new file in database if it does not exist
    /// Returns file id
    fn create_file(&mut self, file: &Path) -> Result<i32, DatabaseError> {
        match self.get_file_id(file) {
            Ok(file_id) => Ok(file_id),
            Err(DatabaseError::FileNotFound(_)) => {
                // Create new file in db if file does not exist
                let path_str = file.to_string_lossy().to_string();
                let new_id: models::File = diesel::insert_into(models::files::table)
                    .values(models::files::path.eq(path_str))
                    .get_result(&mut self.connection)?;
                Ok(new_id.id)
            },
            x => x
        }
    }


    /// Add tag to file
    pub fn add_tag(&mut self, file: &Path, tag: &String) -> Result<(), DatabaseError> {
        let new_tag = models::Tag {
            file_id: self.create_file(file)?,
            tag: tag.clone(),
        };
        let result = diesel::insert_into(models::tags::table)
            .values(&new_tag)
            .execute(&mut self.connection);
        match result {
            Ok(_) => Ok(()),
            // Ignore error when trying to add the same tag twice
            Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => Ok(()),
            Err(e) => Err(e)?
        }
    }


    /// Add attribute to file
    pub fn add_attribute(&mut self, file: &Path, key: String, value: String) -> Result<(), DatabaseError> {
        let new_attribute = models::Attribute {
            file_id: self.create_file(file)?,
            attr_key: key,
            attr_value: value
        };
        let result = diesel::insert_into(models::attributes::table)
            .values(&new_attribute)
            .execute(&mut self.connection);
        match result {
            Ok(_) => Ok(()),
            // Ignore error when trying to add the same tag twice
            Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => Ok(()),
            Err(e) => Err(e)?
        }
    }


    /// Remove tag from file
    pub fn remove_tag(&mut self, file: &Path, tag: &Tag) -> Result<(), DatabaseError> {
        let file_id = self.get_file_id(file)?;
        match tag {
            Tag::Key(key) => {
                let db_tag = models::tags::table
                    .filter(models::tags::file_id.is(file_id))
                    .filter(models::tags::tag.is(key));
                diesel::delete(db_tag)
                    .execute(&mut self.connection)?;
            },
            Tag::KeyValue { key, value } => {
                let db_attribute = models::attributes::table
                    .filter(models::attributes::file_id.is(file_id))
                    .filter(models::attributes::attr_key.is(key))
                    .filter(models::attributes::attr_value.is(value));
                diesel::delete(db_attribute)
                    .execute(&mut self.connection)?;
            }
        }
        Ok(())
    }


    /// Remove file with all tags and attributes from database
    pub fn remove_file(&mut self, file: &Path) -> Result<(), DatabaseError> {
        let file_id = self.get_file_id(file)?;
        let file = models::files::table
            .filter(models::files::id.is(file_id));
        diesel::delete(file)
            .execute(&mut self.connection)?;
        Ok(())
    }


    fn get_files(&mut self) -> Result<Vec<(i32, String)>, DatabaseError> {
        let files = models::files::table
            .select((models::files::id, models::files::path))
            .load::<(i32, String)>(&mut self.connection)?;
        Ok(files)
    }


    /// Returns an iterator with all files
    pub fn get_all_files(&mut self) -> Result<Vec<SearchResult>, DatabaseError> {
        let results = self.get_files()?
            .into_iter()
            .filter_map(|(file_id, file_path)| {
                self.get_file(file_id, PathBuf::from_str(&file_path).ok()?).ok()
            })
            .collect();
        Ok(results)
    }


    fn get_file(&mut self, file_id: i32, file_path: PathBuf) -> Result<SearchResult, DatabaseError> {
        let tags: HashSet<String> = models::tags::table
            .filter(models::tags::file_id.is(file_id))
            .select(models::tags::tag)
            .load::<String>(&mut self.connection)?
            .into_iter()
            .collect();
        let attributes = models::attributes::table
            .filter(models::attributes::file_id.is(&file_id))
            .select((models::attributes::attr_key, models::attributes::attr_value))
            .load::<(String, String)>(&mut self.connection)?;
        return Ok(SearchResult {
            path: file_path,
            tags,
            attributes,
        });
    }


    /// Get information about file
    pub fn get_file_from_path(&mut self, file_path: &Path) -> Result<SearchResult, DatabaseError> {
        let file_id = self.get_file_id(file_path)?;
        self.get_file(file_id, file_path.to_path_buf())
    }


    /// Search for files matching `search_term`
    pub fn search(&mut self, search_term: SearchExpression) -> Result<Vec<SearchResult>, DatabaseError> {
        let results = self.get_all_files()?
            .into_iter()
            .filter(|result| match_search_query(&result, &search_term))
            .collect();
        Ok(results)
    }


    /// Move all data about `original_path` to `new_path`.
    /// This will not move the file on disk.
    pub fn move_file(&mut self, original_path: &Path, new_path: PathBuf) -> Result<(), DatabaseError>  {
        let original_str = original_path.to_string_lossy().to_string();
        let new_str = new_path.to_string_lossy().to_string();
        diesel::update(models::files::table)
            .filter(models::files::path.is(original_str))
            .set(models::files::path.eq(new_str))
            .execute(&mut self.connection)?;
        Ok(())
    }

}


/// Returns true if `filedata` matches `search_term`
fn match_search_query(result: &SearchResult, search_term: &SearchExpression) -> bool {
    match search_term {
        SearchExpression::Tag(tag) => result.tags.contains(tag),
        SearchExpression::Attribute { key, value } => result.has_attribute(key, value),
        SearchExpression::BinaryOp{ left, right, op_type } => {
            match op_type {
                BinaryOp::And =>
                    match_search_query(result, &left) && match_search_query(result, &right),
                BinaryOp::Or =>
                    match_search_query(result, &left) || match_search_query(result, &right),
            }
        },
        SearchExpression::UnaryOp{ expr, op_type } => {
            match op_type {
                UnaryOp::Not => !match_search_query(result, &expr)
            }
        },
        SearchExpression::Empty => true
    }
}

impl SearchResult {

    pub fn has_attribute(&self, key: &Option<String>, value: &Option<String>) -> bool {
        match (key, value) {
            (Some(key), None) => self.attributes
                .iter()
                .filter(|(x, _)| key == x)
                .next()
                .is_some(),
            (Some(key), Some(value)) =>
                self.attributes
                    .iter()
                    .find(|(x, y)| key == x && value == y)
                    .is_some(),
            (None, Some(value)) =>
                self.attributes
                    .iter()
                    .find(|(_, x)| value == x)
                    .is_some(),
            (None, None) => true,
        }
    }

}

#[cfg(test)]
mod test {

    use super::super::Tag;
    use std::str::FromStr;
    use diesel::Connection;
    use diesel_migrations::MigrationHarness;

    fn create_memory_db() -> super::DatabaseStorage {
        let connection = diesel::sqlite::SqliteConnection::establish(":memory:")
            .unwrap();
        let mut data = super::DatabaseStorage { connection };
        data.connection.revert_all_migrations(super::MIGRATIONS).unwrap();
        data.connection.run_pending_migrations(super::MIGRATIONS).unwrap();
        return data;
    }

    fn file_contains(data: &mut super::DatabaseStorage, path: &super::Path, tag: &String) -> bool {
        data.get_file_from_path(path).unwrap().tags.contains(tag)
    }

    #[test]
    fn add_tag() {
        let mut data = create_memory_db();
        let path = std::path::PathBuf::from_str("test_file").unwrap();
        let tag = "test_tag".to_string();
        data.add_tag(&path, &tag).unwrap();
        assert!(file_contains(&mut data, &path, &tag));
    }

    #[test]
    fn remove_tag() {
        let mut data = create_memory_db();
        let path = std::path::PathBuf::from_str("test_file").unwrap();
        let tag = "test_tag".to_string();
        data.add_tag(&path, &tag).unwrap();
        data.remove_tag(&path, &Tag::new(&tag)).unwrap();
        assert!(!file_contains(&mut data, &path, &tag));
    }

    #[test]
    fn remove_file() {
        let mut data = create_memory_db();
        let path = std::path::PathBuf::from_str("test_file").unwrap();
        let tag = "test_tag".to_string();
        data.add_tag(&path, &tag).unwrap();
        data.remove_file(&path).unwrap();
        assert!(data.get_file_from_path(&path).is_err());
    }

    #[test]
    fn search() {
        let mut data = create_memory_db();
        let path = std::path::PathBuf::from_str("test_file").unwrap();
        let tag = "test_tag".to_string();
        data.add_tag(&path, &tag).unwrap();
        assert_eq!(
            data.search(crate::search::parse("test_tag").unwrap()).unwrap()[0].path,
            path
        );
        assert!(data.search(crate::search::parse("test_tag2").unwrap()).unwrap().is_empty());
    }

}
