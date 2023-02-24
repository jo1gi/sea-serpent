use crate::search::{SearchExpression, UnaryOp, BinaryOp};

use super::{Database, data::FileData};

use std::path::{PathBuf, Path};
use std::collections::{HashSet, HashMap};

#[derive(serde::Serialize)]
pub struct SearchResult<'a> {
    pub path: &'a PathBuf,
    pub tags: &'a HashSet<String>,
    pub attributes: &'a HashMap<String, Vec<String>>,
}

impl Database {

    pub fn search(&self, search_term: SearchExpression) -> Vec<SearchResult> {
        self.data.get_files()
            .filter(|(_path, filedata)| match_search_query(&filedata, &search_term))
            .map(|(path, filedata)| SearchResult {
                path,
                tags: &filedata.tags,
                attributes: &filedata.attributes
            })
            .collect()
    }

    pub fn get_file_info(&self, file: &Path) -> Result<SearchResult, super::DatabaseError> {
        let relative_path = super::find::path_relative_to_db_root(file, &self.root_dir()?)?;
        self.data.get_file(&relative_path)
            .map(|(path, filedata)| SearchResult {
                path,
                tags: &filedata.tags,
                attributes: &filedata.attributes,
            })
            .ok_or_else(|| super::DatabaseError::FileNotFound(file.to_path_buf()))
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
