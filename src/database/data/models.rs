use diesel::prelude::*;

#[derive(Insertable)]
#[diesel(table_name = files)]
pub struct NewFile {
    pub path: String
}

#[derive(Queryable, Insertable, Selectable)]
#[diesel(table_name = files)]
pub struct File {
    pub id: i32,
    pub path: String
}

diesel::table! {
    files (id) {
        id -> Integer,
        path -> Text,
    }
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = tags)]
pub struct Tag {
    pub tag: String,
    pub file_id: i32,
}

diesel::table! {
    tags (file_id, tag) {
        file_id -> Integer,
        tag -> Text,
    }
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = attributes)]
pub struct Attribute {
    pub file_id: i32,
    pub attr_key: String,
    pub attr_value: String
}

diesel::table! {
    attributes (file_id, attr_key, attr_value) {
        file_id -> Integer,
        attr_key -> Text,
        attr_value -> Text,
    }
}

joinable!(tags -> files (file_id));
joinable!(attributes -> files (file_id));
allow_tables_to_appear_in_same_query!(files, tags);
allow_tables_to_appear_in_same_query!(files, attributes);
allow_tables_to_appear_in_same_query!(tags, attributes);
