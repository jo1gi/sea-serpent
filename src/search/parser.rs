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
    Attribute {
        key: Option<String>,
        value: Option<String>
    },
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
        LexItem::Word(s) => parse_word(s.clone(), iter),
        LexItem::Not => parse_unary(UnaryOp::Not, iter),
        LexItem::Or | LexItem::EndParen => Err(ParseError::UnexpectedToken),
        LexItem::StartParen => parse_paren(iter),
        LexItem::AttributeSeperator => parse_attribute(None, iter),
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

fn parse_word(word: String, iter: &mut Tokens) -> Result<Expression, ParseError> {
    if let Some(LexItem::AttributeSeperator) = iter.peek() {
        iter.next();
        parse_attribute(Some(word), iter)
    } else {
        parse_expr(Expression::Tag(word.clone()), iter)
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

fn parse_attribute(key: Option<String>, iter: &mut Tokens) -> Result<Expression, ParseError> {
    let next = iter.peek();
    let value = if let Some(LexItem::Word(value)) = next {
        Some(value.clone())
    } else {
        None
    };
    if value.is_some() || key.is_some() {
        iter.next();
        parse_expr(Expression::Attribute { key, value }, iter)
    } else {
        parse_next(iter)
    }
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
            parse(vec![LexItem::word("tag")]).unwrap(),
            Expression::tag("tag")
        )
    }

    #[test]
    fn two_tags() {
        assert_eq!(
            parse(vec![LexItem::word("A"), LexItem::word("B")]).unwrap(),
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
            parse(vec![LexItem::word("A"), LexItem::Or, LexItem::word("B")]).unwrap(),
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
            parse(vec![LexItem::StartParen, LexItem::word("tag"), LexItem::EndParen]).unwrap(),
            Expression::tag("tag")
        )
    }

    #[test]
    fn paren_with_trailing() {
        assert_eq!(
            parse(vec![
                LexItem::StartParen, LexItem::word("A"), LexItem::EndParen, LexItem::word("B")
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
            parse(vec![LexItem::Not, LexItem::word("tag")]).unwrap(),
            Expression::UnaryOp{
                expr: Box::new(Expression::tag("tag")),
                op_type: super::UnaryOp::Not
            }
        )
    }

    #[test]
    fn attribute() {
        assert_eq!(
            parse(vec![LexItem::word("key"), LexItem::AttributeSeperator, LexItem::word("value")]).unwrap(),
            Expression::Attribute {
                key: Some("key".to_string()),
                value: Some("value".to_string())
            }
        )
    }

    #[test]
    fn unary_catches_single() {
        assert_eq!(
            parse(vec![LexItem::Not, LexItem::word("A"), LexItem::word("B")]).unwrap(),
            Expression::BinaryOp {
                op_type: BinaryOp::And,
                left: Box::new(Expression::UnaryOp{
                    expr: Box::new(Expression::tag("A")),
                    op_type: super::UnaryOp::Not
                }),
                right: Box::new(Expression::tag("B"))
            }
        )
    }
}
