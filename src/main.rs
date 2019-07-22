extern crate chrono;
extern crate hmac;
extern crate reqwest;
extern crate sha2;
extern crate structopt;
extern crate tempfile;

use chrono::{Datelike, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use structopt::StructOpt;
use tempfile::NamedTempFile;
use serde::{Serialize, Deserialize};

type HmacSha256 = Hmac<Sha256>;

const TITLE_BEGIN: &str = "████████████████████████████████████████
██████████████   TITLE   ███████████████
████████████████████████████████████████";
const CONTENT_BEGIN: &str = "████████████████████████████████████████
██████████████  CONTENT  ███████████████
████████████████████████████████████████";
const DATE_BEGIN: &str = "████████████████████████████████████████
██████████████   DATE    ███████████████
████████████████████████████████████████";
// const TAGS_BEGIN: &str = "████████████████████████████████████████
// ██████████████   TAGS    ███████████████
// ████████████████████████████████████████";
const END: &str = "\n";

#[derive(Serialize, Deserialize, Debug)]
struct Article {
    article: ArticleData
}

#[derive(Serialize, Deserialize, Debug)]
struct ArticleData {
    content: String,
    title: String,
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "blawg",
    about = "Create, edit, and publish articles to the web from the safety of your favorite plain text editor."
)]
struct Opt {
    #[structopt(parse(from_os_str), help = "Path to a file. The content will be loaded as the content of an article.")]
    input: Option<PathBuf>,
    #[structopt(short = "u", long = "update", help = "Update an existing article")]
    slug: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {


    // Check for required ENV variables

    let _api_create_url = match std::env::var("BLAWG_API_CREATE") {
        Ok(url) => url,
        Err(_e) => {
            eprintln!("Error: 'BLAWG_API_CREATE' environment variable undefined.");
            std::process::exit(1);
        }
    };


    let opt = Opt::from_args();

    let slug = match opt.slug {
        Some(slug) => slug,
        None => String::from("")
    };

    let api_update_url = match std::env::var("BLAWG_API_UPDATE") {
        Ok(url) => url.replace("{slug}", &slug),
        Err(_e) => {
            eprintln!("Error: 'BLAWG_API_UPDATE' environment variable undefined.");
            std::process::exit(1);
        }
    };

    println!("{} {}", &slug, &api_update_url.replace("{slug}", &slug));

    let file = NamedTempFile::new()?;

    if slug != "" {
        let article: Article = match load_remote_content(&slug) {
            Ok(article) => article,
            Err(_e) => {
                eprintln!("Error: Article matching '{}' not found.", &slug);
                std::process::exit(1);
            },
        };

        let path = file.path();
        std::fs::write(path, build_defaults_from_remote(article))?;
    } else {
        // Prepare contents of temporary file

        let existing_content = match opt.input {
            Some(file_path) => std::fs::read_to_string(file_path)?,
            None => String::from(""),
        };

        let path = file.path();
        std::fs::write(path, build_defaults(&existing_content))?;
    }


    // Spawn text editor to to open a new document
    // which is formatted correctly, and has appropriate
    // content.

    let text_editor_cmd = match std::env::var("BLAWG_EDITOR") {
        Ok(cmd) => cmd,
        Err(_e) => {
            eprintln!("Error: 'BLAWG_EDITOR' environment variable undefined.");
            std::process::exit(1);
        }
    };

    let child = match Command::new(&text_editor_cmd).arg(file.path()).spawn() {
        Ok(child) => child,
        Err(e) => {
            eprintln!("Error: Could not open text editor.\n{:?}", e);
            std::process::exit(1);
        }
    };


    // Collect output from file when editor is closed
    // and create a request payload.

    let _output = child.wait_with_output()
        .expect("Failed to wait on child process");
    let content_payload = std::fs::read_to_string(file.path())?;
    let request = create_request_payload(&content_payload);


    // Hash the contents of the message and
    // send the request.

    let date = request.get("article").expect("no article").get("date");
    let content = request.get("article").expect("no article").get("content");
    let title = request.get("article").expect("no article").get("title");
    // let tags = request.get("tags");

    if content.is_none() || content == Some(&String::from("")) {
        std::process::exit(0);
    }


    // if let (Some(date), Some(content), Some(tags)) = (date, content, tags) {
    if let (Some(date), Some(content), Some(title)) = (date, content, title) {

        let mut response =
            reqwest::Client::new()
            .patch(&api_update_url)
            .json(&request)
            // .header(reqwest::header::AUTHORIZATION, create_request_hmac(&date, &content, &tags))
            .header(reqwest::header::AUTHORIZATION, create_request_hmac(&date, &content, &title))
            .send()?;

        println!("{:?}", response);

        match response.status() {
            reqwest::StatusCode::OK => println!("Posted"),
            unhandled => println!("{:?}", unhandled),
        };

        let errors = match response.json() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("error happened {:?}", e);
                std::process::exit(1);
            }
        };

        println!("{:?}", &errors);
    }

    Ok(())
}

fn extract(content: &String, delim: (&str, &str)) -> String {
    let s = &content[content.find(delim.0).unwrap()..];
    let start = s.trim_start_matches(delim.0);

    if delim.1 == "\n" {
        return String::from(start.trim());
    }

    let ind = start.find(delim.1).unwrap();
    let (result, _junk) = start.split_at(ind);

    String::from(result.trim())
}

fn build_defaults(existing_content: &str) -> String {
    // let default_tags = "";
    let default_title = "";
    let now = Utc::now();
    let (_ce, year) = now.year_ce();
    let default_date = format!("{}/{}/{}", now.month(), now.day(), year);
    let default_content = format!(
        "{begin_title}\n{title}\n\n{begin_content}\n{content}\n\n{begin_date}\n{date}",
        date=default_date,
        title=default_title,
        begin_content=CONTENT_BEGIN,
        content=existing_content,
        begin_date=DATE_BEGIN,
        begin_title=TITLE_BEGIN
    );

    String::from(default_content)
}

fn build_defaults_from_remote(article: Article) -> String {
    format!(
        "{begin_title}\n{title}\n\n{begin_content}\n{content}\n\n{begin_date}\n{date}",
        date=String::from(""),
        title=article.article.title,
        begin_content=CONTENT_BEGIN,
        content=article.article.content,
        begin_date=DATE_BEGIN,
        begin_title=TITLE_BEGIN
    )
}

fn create_request_payload(content_payload: &String) -> HashMap<&str, HashMap<&str, String>> {
    let content = extract(content_payload,
                          (CONTENT_BEGIN, DATE_BEGIN));
    let date = extract(content_payload,
                          (DATE_BEGIN, END));
    let title = extract(content_payload,
                          (TITLE_BEGIN, CONTENT_BEGIN));

    let mut request = HashMap::new();
    let mut inner = HashMap::new();
    inner.insert("content", content);
    inner.insert("date", date);
    // request.insert("tags", tags);
    inner.insert("title", title);

    request.insert("article", inner);

    request
}

fn create_request_hmac(date: &String, content: &String, title: &String) -> String {
    let secret_key = match std::env::var("BLAWG_SECRET_KEY") {
        Ok(key) => key,
        Err(_e) => {
            eprintln!("Error: 'BLAWG_SECRET_KEY' environment variable undefined.");
            std::process::exit(1);
        }
    };

    let mut mac = HmacSha256::new_varkey(secret_key.as_bytes())
        .expect("HMAC can take key of any size");

    let mut all_params = vec![
      "article",
      "content",
      "title",
      "date",
      &date,
      &content,
      &title
    ];

    all_params.sort();

    let serialized_params = all_params.join("");

    println!("{}", serialized_params);
    mac.input(serialized_params.as_bytes());

    let result = mac.result();
    let code_bytes = result.code();

    format!(
        "hmac {}",
        code_bytes
            .iter()
            .map(|x| format!("{:02x}", x))
            .fold(String::new(), |acc, x| { acc + &x })
    )
}

fn load_remote_content(slug: &String) -> Result<Article, reqwest::Error> {
    let path_var = match std::env::var("BLAWG_API_GET") {
        Ok(p) => p,
        Err(_e) => {
            eprintln!("Error: 'BLAWG_API_GET' environment variable undefined.");
            std::process::exit(1);
        }
    };
    let path = format!("{}/{}", path_var, slug);
    let response_body: Article = reqwest::get(&path)?.json()?;

    Ok(response_body)
}
