pub mod qry;
pub mod apploader;

pub fn query_snippets(engine : &mut qry::QueryEngine, query : &str, return_ids : bool, folder : Option<&std::string::String>, tag : Option<&std::string::String>) -> String {
    let mut rquery = query.to_string();

    if let Some(folder) = folder {
        // query folders and return ids
        let folder_ids = query_folders(engine, folder, true);
        if !folder_ids.is_empty() {
            // add js query contains
            rquery = format!("{}.includes(snippet.folderId) && {}", folder_ids, rquery);
        }
    }
    
    if let Some(tag) = tag {
        // query tags and return ids
        let tag_ids = query_tags(engine, tag, true);
        if !tag_ids.is_empty() {
            rquery = format!("(snippet.tagsIds.some(tagId => {}.includes(tagId))) && {}", tag_ids, rquery);
        }
    }

    match engine.query("snippet", &rquery, Some("snippets"), return_ids) {
        Ok(output) => output,
        Err(e) => e
    }
}

pub fn query_folders(engine : &mut qry::QueryEngine, query : &str, return_ids : bool) -> String {
    match engine.query("folder", query, Some("folders"), return_ids) {
        Ok(output) => output,
        Err(e) => e
    }
}

pub fn query_tags(engine : &mut qry::QueryEngine, query : &str, return_ids : bool) -> String {
    match engine.query("tag", query, Some("tags"), return_ids) {
        Ok(output) => output,
        Err(e) => e
    }
}