use std::iter::Peekable;

use thiserror::Error;
use displaydoc::Display;

#[derive(Debug, Error, Display)]
pub enum LexError {
    /// Invalid char {0}
    InvalidChar(char)
}

#[derive(Debug, PartialEq)]
pub enum LexItem {
    Tag(String),
    StartParen,
    EndParen,
    Or,
    Not,
}

pub fn lex(input: &str) -> Result<Vec<LexItem>, LexError> {
    let mut result = Vec::new();

    let mut it = input.chars().peekable();
    while let Some(c) = it.peek() {
        let item = match c {
            '(' => { it.next(); Some(LexItem::StartParen) },
            ')' => { it.next(); Some(LexItem::EndParen) },
            ' ' => { it.next(); None },
            _  => {
                let word = get_word(&mut it);
                match word.as_str() {
                    "or" => Some(LexItem::Or),
                    "not" => Some(LexItem::Not),
                    "and" => None,
                    _ => Some(LexItem::Tag(word)),
                }
            },
        };
        if let Some(item) = item {
            result.push(item);
        }
    }

    return Ok(result);
}

fn get_word<T: Iterator<Item = char>>(iter: &mut Peekable<T>) -> String {
    let mut quoted_string = false;
    iter.take_while(|c| {
        if *c == '"' {
            quoted_string = !quoted_string;
        }
        (*c != ' ' && *c != '(' && *c != ')') || quoted_string
    }).filter(|c| *c != '"')
        .collect()
}

#[cfg(test)]
mod test {
    use super::{get_word, lex, LexItem};

    impl LexItem {
        pub fn tag(s: &str) -> Self {
            Self::Tag(s.to_string())
        }
    }

    fn test_get_word(s: &str, result: &str) {
        let mut it = s.chars().peekable();
        assert_eq!(&get_word(&mut it), result);
    }


    #[test]
    fn lex_word() {
        assert_eq!(
            lex("word").unwrap(), vec![LexItem::Tag("word".to_string())]
        )
    }

    #[test]
    fn lex_paren() {
        assert_eq!(
            lex("word (word)").unwrap(),
            vec![
                LexItem::Tag("word".to_string()),
                LexItem::StartParen,
                LexItem::Tag("word".to_string()),
                LexItem::EndParen,
            ]
        )
    }

    #[test]
    fn lex_or() {
        assert_eq!(
            lex("A or B").unwrap(),
            vec![LexItem::Tag("A".to_string()), LexItem::Or, LexItem::Tag("B".to_string())]
        )
    }

    #[test]
    fn get_word_simple() {
        test_get_word("tag something else", "tag")
    }

    #[test]
    fn get_word_quoted() {
        test_get_word("\"tag something\" else", "tag something");
    }

}
