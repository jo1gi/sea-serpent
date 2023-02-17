mod lexer;
mod parser;

use thiserror::Error;
use displaydoc::Display;

pub use parser::{
    UnaryOp, BinaryOp,
    Expression as SearchExpression
};

#[derive(Debug, Error, Display)]
pub enum SearchError {
    /// Failed to lex input
    LexError(#[from] lexer::LexError),
    /// Failed to parse tokens
    ParseError(#[from] parser::ParseError),
}

pub fn parse(input: &str) -> Result<SearchExpression, SearchError> {
    let tokens = lexer::lex(input)?;
    let expression = parser::parse(tokens)?;
    Ok(expression)
}
