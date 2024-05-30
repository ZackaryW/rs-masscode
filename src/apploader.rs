use std::{collections::HashMap, fs, path::PathBuf, time::SystemTime};
use serde_json::Value;

pub struct Loader {
    prefs_path: PathBuf,
    last_modified: Option<SystemTime>,
    preferences: Option<HashMap<String, String>>,
}

impl Loader {
    pub fn new(appdata_path: Option<&str>) -> Self {
        // Determine the path for preferences.json based on the provided appdata_path or default platform path
        let prefs_path = appdata_path
            .map(PathBuf::from)
            .unwrap_or_else(Self::get_possible_path_by_platform)
            .join("v2/preferences.json");

        Self {
            prefs_path,
            last_modified: None,
            preferences: None,
        }
    }

    // Helper function to fetch the last modified time of preferences.json
    fn fetch_last_modified(&self) -> SystemTime {
        fs::metadata(&self.prefs_path)
            .and_then(|metadata| metadata.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH)  // Using UNIX_EPOCH as a fallback
    }

    // Load or reload preferences if the file has been modified
    pub fn load_preferences(&mut self) -> Result<&HashMap<String, String>, Box<dyn std::error::Error>> {
        if self.preferences.is_none() || self.prefs_cache_expired() {
            let prefs_str = fs::read_to_string(&self.prefs_path)?;
            let prefs_json: Value = serde_json::from_str(&prefs_str)?;
            let mut prefs_map = HashMap::new();

            if let Some(prefs_obj) = prefs_json.as_object() {
                for (key, val) in prefs_obj {
                    if let Some(str_val) = val.as_str() {
                        prefs_map.insert(key.clone(), str_val.to_string());
                    }
                }
            }

            self.preferences = Some(prefs_map);
        }
        Ok(self.preferences.as_ref().unwrap())
    }

    // Check if the preferences cache is expired
    pub fn prefs_cache_expired(&self) -> bool {
        self.last_modified
            .map(|last_modified| self.fetch_last_modified() > last_modified)
            .unwrap_or(true)
    }

    // Determine the path based on the operating platform
    fn get_possible_path_by_platform() -> PathBuf {
        if cfg!(target_os = "windows") {
            PathBuf::from(env!("APPDATA")).join("massCode")
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/home/".to_string());
            PathBuf::from(home).join(".massCode")
        }
    }

    pub fn load_db_path(mut self) -> String {
        let prefs = self.load_preferences().unwrap();
        let storagepath = prefs.get("storagePath").unwrap();
        // combine with db.json
        format!("{}/db.json", storagepath)
    }
}