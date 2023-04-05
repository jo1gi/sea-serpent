mod models;

use super::DatabaseError;
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
    Connection, RunQueryDsl, QueryDsl,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness,};

/// Name of sqlite file
const DATA_FILE: &'static str = "data.sqlite";

/// Sql migration data
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub struct DatabaseData {
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

impl DatabaseData {

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

    /// Add tag to file
    pub fn add_tag(&mut self, file: &Path, tag: &String) -> Result<(), DatabaseError> {
        let new_tag = models::Tag {
            path: file.to_string_lossy().to_string(),
            tag: tag.clone(),
        };
        diesel::insert_into(models::tags::table)
            .values(&new_tag)
            .execute(&mut self.connection)?;
        Ok(())
    }

    /// Add attribute to file
    pub fn add_attribute(&mut self, file: &Path, key: String, value: String) -> Result<(), DatabaseError> {
        let new_attribute = models::Attribute {
            path: file.to_string_lossy().to_string(),
            attr_key: key,
            attr_value: value
        };
        diesel::insert_into(models::attributes::table)
            .values(&new_attribute)
            .execute(&mut self.connection)?;
        Ok(())
    }

    /// Remove tag from file
    pub fn remove_tag(&mut self, file: &Path, tag: &String) -> Result<(), DatabaseError> {
        let path_str = file.to_string_lossy().to_string();
        let db_tag = models::tags::table
            .filter(models::tags::path.is(path_str))
            .filter(models::tags::tag.is(tag));
        diesel::delete(db_tag)
            .execute(&mut self.connection)?;
        Ok(())
    }

    pub fn remove_attribute(&mut self, file: &Path, key: String, value: String) -> Result<(), DatabaseError> {
        let path_str = file.to_string_lossy().to_string();
        let db_attribute = models::attributes::table
            .filter(models::attributes::path.is(path_str))
            .filter(models::attributes::attr_key.is(key))
            .filter(models::attributes::attr_value.is(value));
        diesel::delete(db_attribute)
            .execute(&mut self.connection)?;
        Ok(())
    }

    /// Remove file from database
    pub fn remove_file(&mut self, file: &Path) -> Result<(), DatabaseError> {
        let path_str = file.to_string_lossy().to_string();
        let tags = models::tags::table
            .filter(models::tags::path.is(&path_str));
        diesel::delete(tags)
            .execute(&mut self.connection)?;
        let attributes = models::attributes::table
            .filter(models::attributes::path.is(&path_str));
        diesel::delete(attributes)
            .execute(&mut self.connection)?;
        Ok(())
    }

    fn get_filenames(&mut self) -> Result<Vec<String>, DatabaseError> {
       let mut tags = models::tags::table
            .select(models::tags::path)
            .distinct()
            .load::<String>(&mut self.connection)?;
       let attributes = models::attributes::table
            .select(models::attributes::path)
            .distinct()
            .load::<String>(&mut self.connection)?;
        tags.extend(attributes);
        tags.sort();
        tags.dedup();
        Ok(tags)

    }

    /// Returns an iterator with all files
    pub fn get_all_files(&mut self) -> Result<Vec<SearchResult>, DatabaseError> {
        let results = self.get_filenames()?
            .into_iter()
            .filter_map(|filename| {
                let tags = models::tags::table
                    .filter(models::tags::path.is(&filename))
                    .select(models::tags::tag)
                    .load::<String>(&mut self.connection)
                    .ok()?;
                let attributes = models::attributes::table
                    .filter(models::attributes::path.is(&filename))
                    .select((models::attributes::attr_key, models::attributes::attr_value))
                    .load::<(String, String)>(&mut self.connection)
                    .ok()?;
                Some(SearchResult {
                    path: PathBuf::from_str(&filename).unwrap(),
                    tags: tags.into_iter().collect(),
                    attributes
                })
            })
            .collect();
        Ok(results)
    }

    /// Get information about file
    pub fn get_file(&mut self, file: &Path) -> Result<SearchResult, DatabaseError> {
        let path_str = file.to_string_lossy().to_string();
        let tags: HashSet<String> = models::tags::table
            .filter(models::tags::path.is(path_str))
            .load::<models::Tag>(&mut self.connection)?
            .into_iter()
            .map(|tag| tag.tag)
            .collect();
        return Ok(SearchResult {
            path: file.to_path_buf(),
            tags,
            attributes: Vec::new(),
        });
    }

    /// Search for files matching `search_term`
    pub fn search(&mut self, search_term: SearchExpression) -> Result<Vec<SearchResult>, DatabaseError> {
        let results = self.get_all_files()?
            .into_iter()
            .filter(|result| match_search_query(&result, &search_term))
            .collect();
        Ok(results)
    }

    /// Move all data about `original_path` to `new_path`
    pub fn move_file(&mut self, original_path: &Path, new_path: PathBuf) -> Result<(), DatabaseError>  {
        let original_str = original_path.to_string_lossy().to_string();
        let new_str = new_path.to_string_lossy().to_string();
        diesel::update(models::tags::table)
            .filter(models::tags::path.is(original_str))
            .set(models::tags::path.eq(new_str))
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

    use std::str::FromStr;
    use diesel::Connection;
    use diesel_migrations::MigrationHarness;

    fn create_memory_db() -> super::DatabaseData {
        let connection = diesel::sqlite::SqliteConnection::establish(":memory:")
            .unwrap();
        let mut data = super::DatabaseData { connection };
        data.connection.revert_all_migrations(super::MIGRATIONS).unwrap();
        data.connection.run_pending_migrations(super::MIGRATIONS).unwrap();
        return data;
    }

    fn file_contains(data: &mut super::DatabaseData, path: &super::Path, tag: &String) -> bool {
        data.get_file(path).unwrap().tags.contains(tag)
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
        data.remove_tag(&path, &tag).unwrap();
        assert!(!file_contains(&mut data, &path, &tag));
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
