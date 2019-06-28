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

type HmacSha256 = Hmac<Sha256>;

const CONTENT_BEGIN: &str = "████████████████████████████████████████
██████████████  CONTENT  ███████████████
████████████████████████████████████████";
const DATE_BEGIN: &str = "████████████████████████████████████████
██████████████   DATE    ███████████████
████████████████████████████████████████";
const TAGS_BEGIN: &str = "████████████████████████████████████████
██████████████   TAGS    ███████████████
████████████████████████████████████████";
const TAGS_END: &str = "\n";

#[derive(Debug, StructOpt)]
#[structopt(name = "Blawg", about = "Use plain text editor of choice to manage web content.")]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {


    // Check for required ENV variables

    let api_create_url = match std::env::var("BLAWG_API_CREATE") {
        Ok(url) => url,
        Err(_e) => {
            eprintln!("Error: 'BLAWG_API_CREATE' environment variable undefined.");
            std::process::exit(1);
        }
    };

    let opt = Opt::from_args();


    // Prepare contents of temporary file

    let existing_content = match opt.input {
        Some(file_path) => std::fs::read_to_string(file_path)?,
        None => String::from(""),
    };

    let file = NamedTempFile::new()?;
    let path = file.path();
    std::fs::write(path, build_defaults(&existing_content))?;


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

    let date = request.get("date");
    let content = request.get("content");
    let tags = request.get("tags");

    if let (Some(date), Some(content), Some(tags)) = (date, content, tags) {

        let response =
            reqwest::Client::new()
            .post(&api_create_url)
            .json(&request)
            .header(reqwest::header::AUTHORIZATION, create_request_hmac(&date, &content, &tags))
            .send()?;


        match response.status() {
            reqwest::StatusCode::OK => println!("Posted"),
            unhandled => println!("{:?}", unhandled),
        };
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
    let default_tags = "";
    let now = Utc::now();
    let (_ce, year) = now.year_ce();
    let default_date = format!("{}/{}/{}", now.month(), now.day(), year);
    let default_content = format!(
        "{begin_content}\n{content}\n\n{begin_date}\n{date}\n\n{begin_tags}\n{tags}{end_tags}",
        date=default_date,
        tags=default_tags,
        begin_content=CONTENT_BEGIN,
        content=existing_content,
        begin_date=DATE_BEGIN,
        begin_tags=TAGS_BEGIN,
        end_tags=TAGS_END
    );

    String::from(default_content)
}

fn create_request_payload(content_payload: &String) -> HashMap<&str, String> {
    let content = extract(content_payload,
                          (CONTENT_BEGIN, DATE_BEGIN));
    let date = extract(content_payload,
                          (DATE_BEGIN, TAGS_BEGIN));
    let tags = extract(content_payload,
                          (TAGS_BEGIN, TAGS_END));

    let mut request = HashMap::new();
    request.insert("content", content);
    request.insert("date", date);
    request.insert("tags", tags);

    request
}

fn create_request_hmac(date: &String, content: &String, tags: &String) -> String {
    let mut mac = HmacSha256::new_varkey(b"secret_key")
        .expect("HMAC can take key of any size");
    mac.input(format!("{}{}{}", date, content, tags).as_bytes());
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
