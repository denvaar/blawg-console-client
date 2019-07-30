extern crate hmac;
extern crate sha2;

use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::article_type;

type HmacSha256 = Hmac<Sha256>;

pub fn create_request_hmac(article: &article_type::Article, param_keys: Vec<&str>) -> String {
    let secret_key = match std::env::var("BLAWG_SECRET_KEY") {
        Ok(key) => key,
        Err(_e) => {
            eprintln!("Error: 'BLAWG_SECRET_KEY' environment variable undefined.");
            std::process::exit(1);
        }
    };

    let mut mac = HmacSha256::new_varkey(secret_key.as_bytes())
        .expect("HMAC can take key of any size");

    let mut all_params: Vec<&str> = vec![
      &article.article.content,
      &article.article.title
    ];

    all_params.extend(param_keys);

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

