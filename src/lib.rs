pub mod qry;
pub mod apploader;

pub fn query_snippets(engine : &mut qry::QueryEngine, query : &str) -> String {
    match engine.query("snippet", query, Some("snippets")) {
        Ok(output) => output,
        Err(e) => e
    }
}

pub fn query_folders(engine : &mut qry::QueryEngine, query : &str) -> String {
    match engine.query("folder", query, Some("folders")) {
        Ok(output) => output,
        Err(e) => e
    }
}

pub fn query_tags(engine : &mut qry::QueryEngine, query : &str) -> String {
    match engine.query("tag", query, Some("tags")) {
        Ok(output) => output,
        Err(e) => e
    }
}