use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;

use crate::article_type;

const TITLE_BEGIN: &str = "████████████████████████████████████████
██████████████   TITLE   ███████████████
████████████████████████████████████████";
const CONTENT_BEGIN: &str = "████████████████████████████████████████
██████████████  CONTENT  ███████████████
████████████████████████████████████████";
const DATE_BEGIN: &str = "████████████████████████████████████████
██████████████   DATE    ███████████████
████████████████████████████████████████";
const END: &str = "\n";


pub fn open_editor_with_file(existing_file: Option<PathBuf>) -> article_type::Article {
    // TODO: fix duplication between other function and this
    let text_editor_cmd = get_text_editor_command();
    let file = NamedTempFile::new().unwrap();
    let mut original_content = match existing_file {
        Some(path) => std::fs::read_to_string(path).unwrap(),
        None => String::from(""),
    };

    original_content = format_file_content_to_article(original_content);

    std::fs::write(file.path(), &original_content).unwrap();

    let child = match Command::new(&text_editor_cmd).arg(file.path()).spawn() {
        Ok(child) => child,
        Err(e) => {
            eprintln!("Error: Could not open text editor.\n{:?}", e);
            std::process::exit(1);
        }
    };

    let _output = child.wait_with_output()
        .expect("Failed to wait on child process");

    let edited_content = std::fs::read_to_string(file.path()).unwrap();

    if original_content == edited_content {
        println!("No changes");
        std::process::exit(0);
    }

    string_to_article(&edited_content)

}

pub fn open_editor(article: article_type::Article) -> article_type::Article {
    let text_editor_cmd = get_text_editor_command();
    let file = NamedTempFile::new().unwrap();
    let original_content = format_article_for_file(article);

    std::fs::write(file.path(), &original_content).unwrap();

    let child = match Command::new(&text_editor_cmd).arg(file.path()).spawn() {
        Ok(child) => child,
        Err(e) => {
            eprintln!("Error: Could not open text editor.\n{:?}", e);
            std::process::exit(1);
        }
    };

    let _output = child.wait_with_output()
        .expect("Failed to wait on child process");

    let edited_content = std::fs::read_to_string(file.path()).unwrap();

    if original_content == edited_content {
        println!("No changes");
        std::process::exit(0);
    }

    string_to_article(&edited_content)
}

fn string_to_article(article_data: &String) -> article_type::Article {
    let content = extract(article_data,
                          (CONTENT_BEGIN, DATE_BEGIN));
    let _date = extract(article_data,
                          (DATE_BEGIN, END));
    let title = extract(article_data,
                          (TITLE_BEGIN, CONTENT_BEGIN));

    article_type::Article { article: article_type::ArticleData { content, title } }
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

fn format_file_content_to_article(content: String) -> String {
    format!(
        "{begin_title}\n{title}\n\n{begin_content}\n{content}\n\n{begin_date}\n{date}",
        date=String::from(""),
        title=String::from(""),
        begin_content=CONTENT_BEGIN,
        content=content,
        begin_date=DATE_BEGIN,
        begin_title=TITLE_BEGIN
    )
}

fn format_article_for_file(article: article_type::Article) -> String {
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

fn get_text_editor_command() -> String {
    match std::env::var("BLAWG_EDITOR") {
        Ok(cmd) => cmd,
        Err(_e) => {
            eprintln!("Error: 'BLAWG_EDITOR' environment variable undefined.");
            std::process::exit(1);
        }
    }
}

