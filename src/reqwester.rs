extern crate serde_json;

use serde_json::Value;
use crate::article_type;

pub fn fetch_article_content(slug: &String) -> article_type::Article {
    let path_var = match std::env::var("BLAWG_API_GET") {
        Ok(p) => p,
        Err(_e) => {
            eprintln!("Error: 'BLAWG_API_GET' environment variable undefined.");
            std::process::exit(1);
        }
    };
    let path = format!("{}/{}", path_var, slug);
    reqwest::get(&path).unwrap().json().unwrap()
}

pub fn create_article(article_data: article_type::Article, hmac_header_value: String) -> String {
    let api_create_url = match std::env::var("BLAWG_API_CREATE") {
        Ok(url) => url,
        Err(_e) => {
            eprintln!("Error: 'BLAWG_API_CREATE' environment variable undefined.");
            std::process::exit(1);
        }
    };

    let mut response =
        reqwest::Client::new()
        .post(&api_create_url)
        .json(&article_data.article)
        .header(reqwest::header::AUTHORIZATION, hmac_header_value)
        .send()
        .unwrap();

    println!("{:?}", response);

    match response.status() {
        reqwest::StatusCode::OK => println!("Updated"),
        unhandled => println!("{:?}", unhandled),
    };

    let body: Value = match response.json() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error happened {:?}", e);
            std::process::exit(1);
        }
    };

    println!("{:?}", body);

    String::from("complete")
}

pub fn delete_article(slug: String, hmac_header_value: String) -> String {
    let api_delete_url = match std::env::var("BLAWG_API_DELETE") {
        Ok(url) => url.replace("{slug}", &slug),
        Err(_e) => {
            eprintln!("Error: 'BLAWG_API_DELETE' environment variable undefined.");
            std::process::exit(1);
        }
    };

    let mut response =
        reqwest::Client::new()
        .delete(&api_delete_url)
        .header(reqwest::header::AUTHORIZATION, hmac_header_value)
        .send()
        .unwrap();

    println!("{:?}", response);

    match response.status() {
        reqwest::StatusCode::OK => println!("Deleted"),
        unhandled => println!("{:?}", unhandled),
    };

    let body: Value = match response.json() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error happened {:?}", e);
            std::process::exit(1);
        }
    };

    println!("{:?}", body);

    String::from("complete")

}

pub fn update_article(article_data: article_type::Article, slug: String, hmac_header_value: String) -> String {
    let api_update_url = match std::env::var("BLAWG_API_UPDATE") {
        Ok(url) => url.replace("{slug}", &slug),
        Err(_e) => {
            eprintln!("Error: 'BLAWG_API_UPDATE' environment variable undefined.");
            std::process::exit(1);
        }
    };

    let mut response =
        reqwest::Client::new()
        .patch(&api_update_url)
        .json(&article_data)
        .header(reqwest::header::AUTHORIZATION, hmac_header_value)
        .send()
        .unwrap();

    println!("{:?}", response);

    match response.status() {
        reqwest::StatusCode::OK => println!("Updated"),
        unhandled => println!("{:?}", unhandled),
    };

    let body: Value = match response.json() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error happened {:?}", e);
            std::process::exit(1);
        }
    };

    println!("{:?}", body);

    String::from("complete")
}
