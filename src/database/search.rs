use crate::search::{SearchExpression, UnaryOp, BinaryOp};

use super::{Database, data::FileData};

use std::path::PathBuf;
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
