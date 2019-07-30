use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Article {
    pub article: ArticleData
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArticleData {
    pub content: String,
    pub title: String,
}
