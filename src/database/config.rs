use std::path::Path;

#[derive(Default, Debug)]
pub struct DatabaseConfig {

}

fn load_database_config(_database_dir: &Path) -> Option<DatabaseConfig> {
    None
}

pub fn get_database_config(database_dir: &Path) -> DatabaseConfig {
    load_database_config(database_dir)
        .unwrap_or_else(Default::default)
}
