use diesel::prelude::*;

#[derive(Queryable, Insertable)]
#[diesel(table_name = tags)]
pub struct Tag {
    pub path: String,
    pub tag: String
}

diesel::table! {
    tags (path, tag) {
        path -> Text,
        tag -> Text,
    }
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = attributes)]
pub struct Attribute {
    pub path: String,
    pub attr_key: String,
    pub attr_value: String
}

diesel::table! {
    attributes (path, attr_key, attr_value) {
        path -> Text,
        attr_key -> Text,
        attr_value -> Text,
    }
}
