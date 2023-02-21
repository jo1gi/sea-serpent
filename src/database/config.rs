use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::Deserialize;

const CONFIG_FILE: &'static str = "config.toml";

/// Configuration for database
#[derive(Default, Debug, Deserialize)]
pub struct DatabaseConfig {
    /// Optional list of allowed tags
    whitelist: Option<Vec<String>>,
    /// Optional list of nonallowed tags
    blacklist: Option<Vec<String>>,
    /// Mappings from alias to tag list
    #[serde(default)]
    aliases: HashMap<String, Vec<String>>,
}

impl DatabaseConfig {

    pub fn get_alias(&self, alias: &String) -> Option<Vec<&String>> {
        self.aliases.get(alias)
            .map(|x| x.iter().collect())
    }

    /// Checks if the tag is in the whitelist
    pub fn tag_allowed(&self, tag: &String) -> bool {
        let matches_whitelist = tag_mathes_list(tag, &self.whitelist, true);
        let matches_blacklist = !tag_mathes_list(tag, &self.blacklist, false);
        matches_whitelist && matches_blacklist
    }


}

/// Create path to config file from database dir
fn create_config_path(database_path: &Path) -> PathBuf {
    database_path.join(CONFIG_FILE)
}

/// Loads config from disk if possible
fn load_database_config(database_dir: &Path) -> Option<DatabaseConfig> {
    let config_path = create_config_path(database_dir);
    let raw_data = std::fs::read_to_string(&config_path).ok()?;
    return toml::from_str(&raw_data).ok();
}

/// Loads config from disk if possible or returns default config
pub fn get_database_config(database_dir: &Path) -> DatabaseConfig {
    load_database_config(database_dir)
        .unwrap_or_else(Default::default)
}

/// Checks if `tag` is in `list`. If `list` is `None` returns `default`
fn tag_mathes_list(tag: &String, list: &Option<Vec<String>>, default: bool) -> bool {
    list.as_ref()
        .map(|inner_list| inner_list.contains(tag))
        .unwrap_or(default)
}

#[cfg(test)]
mod test {

    #[test]
    fn whitelist() {
        let config = super::DatabaseConfig {
            whitelist: Some(vec!["tag_a".to_string()]),
            ..Default::default()
        };
        assert!(config.tag_allowed(&"tag_a".to_string()));
        assert!(!config.tag_allowed(&"tag_b".to_string()));
    }

    #[test]
    fn blacklist() {
        let config = super::DatabaseConfig {
            blacklist: Some(vec!["tag_b".to_string()]),
            ..Default::default()
        };
        assert!(config.tag_allowed(&"tag_a".to_string()));
        assert!(!config.tag_allowed(&"tag_b".to_string()));
    }

}
