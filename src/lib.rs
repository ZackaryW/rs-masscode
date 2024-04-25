use std::{collections::HashMap, fs, path::PathBuf, time::SystemTime};
use serde_json::Value;
pub mod model;
pub mod query;


pub struct Loader {
    db_path: PathBuf,
    last_modified: Option<SystemTime>,
    internal_data: Option<model::StorageData>,
}

impl Loader {

    fn fetch_last_modified(&self) -> SystemTime {
        fs::metadata(&self.db_path())
            .and_then(|metadata| metadata.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH)  // Using UNIX_EPOCH as a fallback
    }

    pub fn db_path(&self) -> PathBuf {
        // return join + db.json
        self.db_path.join("db.json")
    }

    pub fn new(appdata_path: Option<&str>, db_path: Option<&str>) -> Self {
        // Calculate db_path based on the provided options
        let calculated_db_path = if let Some(db_str) = db_path {
            // Directly use the provided db_path if it exists
            //if ends with db.json, strip it
            if db_str.ends_with("db.json") {
                PathBuf::from(&db_str[..db_str.len() - 8])
            } else {
                PathBuf::from(db_str)
            }
        } else {
            // Compute db_path from appdata_path or default platform path
            let appdata_path = appdata_path
                .map(PathBuf::from)
                .unwrap_or_else(Self::get_possible_path_by_platform);

            let prefs_path = appdata_path.join("v2/preferences.json");

            let prefs_str = std::fs::read_to_string(prefs_path).unwrap();
            let prefs: Value = serde_json::from_str(&prefs_str).unwrap();
            PathBuf::from(prefs["storagePath"].as_str().unwrap())
        };

        // Construct and return the Loader instance
        Self {
            db_path: calculated_db_path,
            last_modified: None,
            internal_data: None,
        }
    }

    // Determining the path based on the operating platform
    fn get_possible_path_by_platform() -> PathBuf {
        if cfg!(target_os = "windows") {
            PathBuf::from(env!("APPDATA")).join("massCode")
        } else {
            let home =std::env::var("HOME").unwrap_or_else(|_| "/home/".to_string());
            PathBuf::from(home).join(".massCode")
        }
    }

    pub fn db_raw_content(&self) -> String {
        fs::read_to_string(&self.db_path()).unwrap_or_default()
    }

    pub fn db_cache_expired(&self) -> bool {
        self.last_modified
            .map(|last_modified| {
                self.fetch_last_modified() > last_modified
            })
            .unwrap_or(true)
    }

    pub fn db_content(&mut self) -> Result<&model::StorageData, Box<dyn std::error::Error>> {
        if self.internal_data.is_none() || self.db_cache_expired() {
            let data_str = self.db_raw_content();
            let value: Value = serde_json::from_str(&data_str)?;
            self.update_internal_cache(value)?;
        }
        // Safely return a reference to the internal data
        Ok(self.internal_data.as_ref().unwrap())
    }

    fn update_internal_cache(&mut self, value: Value) -> Result<(), Box<dyn std::error::Error>> {
        let mut folders = HashMap::new();
        let mut tags = HashMap::new();
        let mut snippets = HashMap::new();

        if let Some(array) = value["folders"].as_array() {
            for item in array {
                let folder: model::FolderModel = serde_json::from_value(item.clone())?;
                folders.insert(folder.id.clone(), folder);
            }
        }
        if let Some(array) = value["tags"].as_array() {
            for item in array {
                let tag: model::TagModel = serde_json::from_value(item.clone())?;
                tags.insert(tag.id.clone(), tag);
            }
        }
        if let Some(array) = value["snippets"].as_array() {
            for item in array {
                let snippet: model::SnippetModel = serde_json::from_value(item.clone())?;
                snippets.insert(snippet.id.clone(), snippet);
            }
        }

        self.internal_data = Some(model::StorageData {
            folders,
            tags,
            snippets,
        });

        Ok(())
    }

    pub fn query_folders(&mut self, query: &str) -> Vec<model::FolderModel> {
        let db = self.db_content().unwrap();
        let queryexp = query::parse_query(query);
        let queryres = &queryexp.unwrap().1;
        let folders = &db.folders.clone();
        let res = query::execute_query(folders, queryres);
        res.unwrap()
    }

    pub fn query_tags(&mut self, query: &str) -> Vec<model::TagModel> {
        let db = self.db_content().unwrap();
        let queryexp = query::parse_query(query);
        let queryres = &queryexp.unwrap().1;
        let tags = &db.tags.clone();
        let res = query::execute_query(tags, queryres);
        res.unwrap()
    }

    pub fn query_snippets(&mut self, query: &str) -> Vec<model::SnippetModel> {
        let db = self.db_content().unwrap();
        let queryexp = query::parse_query(query);
        let queryres = &queryexp.unwrap().1;
        let snippets = &db.snippets.clone();
        let res = query::execute_query(snippets, queryres);
        res.unwrap()
    }

}


