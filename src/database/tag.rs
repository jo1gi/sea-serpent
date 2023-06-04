#[derive(Debug)]
/// Store tag either with key or key-value
pub enum Tag {
    /// Basic
    Key(String),
    /// Attribute with key and value
    KeyValue {
        key: String,
        value: String
    }
}

impl Tag {

    pub fn new(s: &str) -> Self {
        match parse_attribute(&s) {
            None => Tag::Key(s.to_string()),
            Some((key, value)) => Tag::KeyValue{ key, value }
        }
    }

}

/// Parses attribute if possible
fn parse_attribute(tag: &str) -> Option<(String, String)> {
    if !tag.contains(":") {
        return None;
    }
    let mut iter = tag.split(":");
    let key = iter.next()?;
    let value = iter.collect();
    return Some((key.to_string(), value));
}


impl ToString for Tag {

    fn to_string(&self) -> String {
        match self {
            Tag::Key(key) => key.clone(),
            Tag::KeyValue { key, value } => {
                format!("{key}:{value}")
            },
        }
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn parse_key_tag() {
        match super::Tag::new("tag_a") {
            super::Tag::Key(_) => (),
            _ => panic!("Should be key tag")
        }
    }

    #[test]
    fn parse_key_value_tag() {
        match super::Tag::new("key:value") {
            super::Tag::KeyValue { key, value } => {
                assert_eq!("key", key);
                assert_eq!("value", value);
            },
            _ => panic!("Should be key-value tag")
        }
    }

}
