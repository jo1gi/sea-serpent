use crate::search::{SearchExpression, UnaryOp, BinaryOp};

use super::Database;

use std::path::PathBuf;
use std::collections::HashSet;

pub type DatabaseResult<'a> = (&'a PathBuf, &'a HashSet<String>);

impl Database {

    pub fn search(&self, search_term: SearchExpression) -> Vec<DatabaseResult> {
        self.get_files()
            .filter(|(_path, tags)| match_search_query(tags, &search_term))
            .collect()
    }

}

fn match_search_query(tags: &HashSet<String>, search_term: &SearchExpression) -> bool {
    match search_term {
        SearchExpression::Tag(tag) => tags.contains(tag),
        SearchExpression::BinaryOp{ left, right, op_type } => {
            match op_type {
                BinaryOp::And =>
                    match_search_query(tags, &left) && match_search_query(tags, &right),
                BinaryOp::Or =>
                    match_search_query(tags, &left) || match_search_query(tags, &right),
            }
        },
        SearchExpression::UnaryOp{ expr, op_type } => {
            match op_type {
                UnaryOp::Not => !match_search_query(tags, &expr)
            }
        }
    }
}
