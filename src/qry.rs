use mini_v8::{MiniV8, Value};
use std::fs;

pub struct QueryEngine {
    dataset_name : String,
    dataset_raw : String,
}

impl QueryEngine {
    pub fn new(dataset_name : &str, data : Option<&str>, path : Option<&str>) -> Result<Self, String> {
        if data.is_none() && path.is_none() {
            return Err("Either data or path must be provided".to_string());
        }

        if data.is_some() && path.is_some() {
            return Err("Only one of data or path can be provided".to_string());
        }
        
        let real_data;
        if data.is_some() {
            real_data = data.unwrap().to_string();
        } else {
            real_data = fs::read_to_string(path.unwrap())
                .map_err(|e| format!("Failed to read JSON file: {}", e))
                .unwrap();
        }
        
        Ok(QueryEngine {
            dataset_name : dataset_name.to_string(),
            dataset_raw : real_data,
        })
    }

    pub fn query(&mut self, unitname : &str, query: &str, scope : Option<&str>, return_ids : bool) -> Result<String, String> {
        // Load JSON data 
        let mv8 = MiniV8::new();
        let js_script: &str = &format!("let {} = {};", self.dataset_name, self.dataset_raw);
        mv8.eval::<_, Value>(js_script)
            .map_err(|e| format!("Failed to initialize JavaScript environment: {}", e))?;

        if let Some(scope) = scope {
            // Split by "/"
            for scope_part in scope.split('/') {
                // if is number
                let scope_query : String;
                if scope_part.chars().all(|c| c.is_numeric()) {
                    scope_query = format!("{} = {}[{}];", self.dataset_name,self.dataset_name, scope_part);
                } else {
                    scope_query = format!("{} ={}.{};", self.dataset_name, self.dataset_name, scope_part);
                }
                mv8.eval::<String, Value>(scope_query)
                    .map_err(|e| format!("Failed to go down scope: {}", e))?;
            }
        }
        // Execute query
        let js_query = format!("JSON.stringify({}.filter({} => {}), null, 2);", self.dataset_name,unitname, query);
        let result: String = mv8.eval(js_query)
            .map_err(|e| format!("Failed to execute query: {}", e))?;

        if return_ids {
            // Parse the result to extract IDs
            let result_value: serde_json::Value = serde_json::from_str(&result)
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;
            let ids: Vec<String> = result_value.as_array()
                .ok_or("Expected an array".to_string())?
                .iter()
                .filter_map(|item| item.get("id").and_then(|id| id.as_str().map(String::from)))
                .collect();
    
            let ids_json = serde_json::to_string(&ids)
                .map_err(|e| format!("Failed to serialize IDs: {}", e))?;
            Ok(ids_json)
        } else {
            // Return the full query result
            Ok(result)
        }
    }
}

