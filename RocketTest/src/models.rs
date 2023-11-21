use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Content {
    pub topic: String,
    pub subtopic: String,
    pub content: String,
}