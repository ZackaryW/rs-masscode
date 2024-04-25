use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FolderModel {
    pub name: String,
    pub index: i32,
    pub parent_id: Option<String>,
    pub defaultLanguage: String,
    pub isOpen: bool,
    pub isSystem: bool,
    pub createdAt: i64,
    pub updatedAt: i64,
    pub id: String,
}
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
    pub label: String,
    pub language: String,
    pub value: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnippetModel {
    pub name: String,
    pub description: Option<String>,
    pub isDeleted: bool,
    pub isFavorites: bool,
    pub folderId: String,
    pub createdAt: i64,
    pub updatedAt: i64,
    pub tagsIds: Vec<String>,
    pub content: Vec<Content>,
    pub id: String,
}


#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TagModel {
    pub name: String,
    pub createdAt: i64,
    pub updatedAt: i64,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StorageData {
    pub folders: HashMap<String, FolderModel>,
    pub tags: HashMap<String, TagModel>,
    pub snippets: HashMap<String, SnippetModel>,
}

