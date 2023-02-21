use std::iter::Peekable;
use super::lexer::LexItem;

use thiserror::Error;
use displaydoc::Display;

#[derive(Debug, Error, Display)]
pub enum ParseError {
    /// Unexpected end of input
    UnexpectedEndOfInput,
    /// Unexpected token
    UnexpectedToken,

}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Empty,
    Tag(String),
    BinaryOp {
        left: Box<Expression>,
        right: Box<Expression>,
        op_type: BinaryOp,
    },
    UnaryOp {
        expr: Box<Expression>,
        op_type: UnaryOp
    },
}

#[derive(Debug, PartialEq)]
pub enum BinaryOp {
    And, Or
}

#[derive(Debug, PartialEq)]
pub enum UnaryOp {
    Not
}

pub fn parse(tokens: Vec<LexItem>) -> Result<Expression, ParseError> {
    if tokens.is_empty() {
        return Ok(Expression::Empty);
    }
    let mut iter = tokens.into_iter().peekable();
    parse_next(&mut iter)
}

type Tokens = Peekable<std::vec::IntoIter<LexItem>>;

/// Parses the next epxression in `iter`
pub fn parse_next(iter: &mut Tokens) -> Result<Expression, ParseError> {
    let current = iter.next().ok_or(ParseError::UnexpectedEndOfInput)?;
    match current {
        LexItem::Tag(s) => parse_expr(Expression::Tag(s.clone()), iter),
        LexItem::Not => parse_unary(UnaryOp::Not, iter),
        LexItem::Or | LexItem::EndParen => Err(ParseError::UnexpectedToken),
        LexItem::StartParen => parse_paren(iter),
    }
}

fn parse_expr(expr: Expression, iter: &mut Tokens) -> Result<Expression, ParseError> {
    let next = iter.peek();
    match next {
        None | Some(LexItem::EndParen) => Ok(expr),
        Some(next) => {
            let op_type = match next {
                LexItem::Or => { iter.next(); BinaryOp::Or },
                _ => BinaryOp::And,
            };
            Ok(Expression::BinaryOp{
                left: Box::new(expr),
                right: Box::new(parse_next(iter)?),
                op_type,
            })
        }
    }
}

fn parse_paren(iter: &mut Tokens) -> Result<Expression, ParseError> {
    let inner_expr = parse_next(iter)?;
    iter.next();
    parse_expr(inner_expr, iter)
}

fn parse_unary(op_type: UnaryOp, iter: &mut Tokens) -> Result<Expression, ParseError> {
    Ok(Expression::UnaryOp{
        expr: Box::new(parse_next(iter)?),
        op_type
    })
}

#[cfg(test)]
mod test {
    use super::{parse, LexItem, Expression, BinaryOp};

    impl Expression {
        pub fn tag(s: &str) -> Self {
            Self::Tag(s.to_string())
        }
    }

    #[test]
    fn single_tag() {
        assert_eq!(
            parse(vec![LexItem::tag("tag")]).unwrap(),
            Expression::tag("tag")
        )
    }

    #[test]
    fn two_tags() {
        assert_eq!(
            parse(vec![LexItem::tag("A"), LexItem::tag("B")]).unwrap(),
            Expression::BinaryOp{
                left: Box::new(Expression::tag("A")),
                right: Box::new(Expression::tag("B")),
                op_type: BinaryOp::And
            }
        )
    }

    #[test]
    fn or() {
        assert_eq!(
            parse(vec![LexItem::tag("A"), LexItem::Or, LexItem::tag("B")]).unwrap(),
            Expression::BinaryOp{
                left: Box::new(Expression::tag("A")),
                right: Box::new(Expression::tag("B")),
                op_type: BinaryOp::Or
            }
        )
    }

    #[test]
    fn paren() {
        assert_eq!(
            parse(vec![LexItem::StartParen, LexItem::tag("tag"), LexItem::EndParen]).unwrap(),
            Expression::tag("tag")
        )
    }

    #[test]
    fn paren_with_trailing() {
        assert_eq!(
            parse(vec![
                LexItem::StartParen, LexItem::tag("A"), LexItem::EndParen, LexItem::tag("B")
            ]).unwrap(),
            Expression::BinaryOp{
                left: Box::new(Expression::tag("A")),
                right: Box::new(Expression::tag("B")),
                op_type: BinaryOp::And
            }
        )
    }

    #[test]
    fn not() {
        assert_eq!(
            parse(vec![LexItem::Not, LexItem::tag("tag")]).unwrap(),
            Expression::UnaryOp{
                expr: Box::new(Expression::tag("tag")),
                op_type: super::UnaryOp::Not
            }
        )
    }
}
