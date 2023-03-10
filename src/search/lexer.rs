use std::iter::Peekable;
use std::str::Chars;

use thiserror::Error;
use displaydoc::Display;

#[derive(Debug, Error, Display)]
pub enum LexError {
    /// Invalid char {0}
    InvalidChar(char)
}

#[derive(Debug, PartialEq)]
pub enum LexItem {
    Word(String),
    AttributeSeperator,
    StartParen,
    EndParen,
    Or,
    Not,
}

const SPECIAL_CHARS: &[char] = &[
    ' ', '(', ')', ',', ':'
];

pub fn lex(input: &str) -> Result<Vec<LexItem>, LexError> {
    let mut result = Vec::new();

    let mut it = input.chars().peekable();
    while let Some(c) = it.peek() {
        let item = match c {
            '(' => { it.next(); Some(LexItem::StartParen) },
            ')' => { it.next(); Some(LexItem::EndParen) },
            ',' => { it.next(); Some(LexItem::Or) },
            ':' => { it.next(); Some(LexItem::AttributeSeperator) },
            ' ' => { it.next(); None },
            _  => {
                let word = get_word(&mut it);
                match word.as_str() {
                    "or" => Some(LexItem::Or),
                    "not" => Some(LexItem::Not),
                    "and" => None,
                    _ => Some(LexItem::Word(word)),
                }
            },
        };
        if let Some(item) = item {
            result.push(item);
        }
    }

    return Ok(result);
}

fn get_word(iter: &mut Peekable<Chars>) -> String {
    let mut quoted_string = false;
    let mut output = String::new();
    while let Some(c) = iter.peek() {
        if SPECIAL_CHARS.contains(c) && !quoted_string {
            break
        }
        let c = iter.next().unwrap();
        if c == '"' {
            quoted_string = !quoted_string;
        } else {
            output.push(c);
        }
    }
    return output;
}

#[cfg(test)]
mod test {
    use super::{get_word, lex, LexItem};

    impl LexItem {
        pub fn word(s: &str) -> Self {
            Self::Word(s.to_string())
        }
    }

    fn test_get_word(s: &str, result: &str) {
        let mut it = s.chars().peekable();
        assert_eq!(&get_word(&mut it), result);
    }


    #[test]
    fn lex_word() {
        assert_eq!(
            lex("word").unwrap(), vec![LexItem::word("word")]
        )
    }

    #[test]
    fn lex_paren() {
        assert_eq!(
            lex("word (word)").unwrap(),
            vec![
                LexItem::Word("word".to_string()),
                LexItem::StartParen,
                LexItem::Word("word".to_string()),
                LexItem::EndParen,
            ]
        )
    }

    #[test]
    fn lex_or_explicit() {
        assert_eq!(
            lex("A or B").unwrap(),
            vec![LexItem::word("A"), LexItem::Or, LexItem::word("B")]
        )
    }

    #[test]
    fn lex_or_with_comma() {
        assert_eq!(
            lex("A, B").unwrap(),
            vec![LexItem::word("A"), LexItem::Or, LexItem::word("B")]
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

    #[test]
    fn attribute() {
        assert_eq!(
            lex("key:value").unwrap(),
            vec![LexItem::word("key"), LexItem::AttributeSeperator, LexItem::word("value")]
        );
    }

}
