use std::path::PathBuf;
use structopt::StructOpt;

mod auth;
mod article_type;
mod reqwester;
mod edit;

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
    #[structopt(short = "d", long = "delete", help = "Delete an article")]
    delete_slug: Option<String>,
    #[structopt(short = "p", long = "publish", help = "Publish an article")]
    publish_slug: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    match opt.delete_slug {
        Some(slug) => delete_article(slug),
        None => ()
    };

    match opt.slug {
        Some(slug) => update_existing_article(slug),
        None => create_new_article(opt.input)
    };

    Ok(())
}

fn delete_article(slug: String) -> () {
    println!("delete slug '{}'", &slug);

    // create auth hash
    let empty_article = article_type::Article {
        article: article_type::ArticleData { content: String::from(""), title: String::from("") }
    };
    let hmac = auth::create_request_hmac(&empty_article, vec![]);
    // create a DELETE request
    let response = reqwester::delete_article(slug, hmac);
    // display response
    println!("{:?}", response);

    std::process::exit(0);
}

fn update_existing_article(slug: String) -> () {
    println!("do an update using slug '{}'", &slug);

    // get existing article with slug
    let mut article: article_type::Article = reqwester::fetch_article_content(&slug);
    // open existing content in text editor
    article = edit::open_editor(article);
    println!("{:?}", article);
    // create auth hash
    let hmac = auth::create_request_hmac(&article, vec!["article", "content", "title"]);
    println!("{:?}", hmac);
    // send a PATCH request
    let response = reqwester::update_article(article, slug, hmac);
    // display response
    println!("{:?}", response);

    std::process::exit(0);
}

fn create_new_article(file_path: Option<PathBuf>) -> () {
    // create new document, or open existing content in text editor
    let article: article_type::Article = edit::open_editor_with_file(file_path);
    // create auth hash
    let hmac = auth::create_request_hmac(&article, vec!["content", "title"]);
    println!("{:?}", hmac);
    // send a POST request
    let response = reqwester::create_article(article, hmac);
    // display response
    println!("{:?}", &response);

    std::process::exit(0);
}
