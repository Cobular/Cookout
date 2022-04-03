use reqwest::{Response, Client};

pub async fn load_page(client: &Client, url: &str) -> reqwest::Result<Response> {
    let res = client.get(url).send().await?;
    Ok(res)
}

pub fn prep_name_for_file(name: &str) -> String {
    name.chars()
        .filter_map(|char| match char {
            x if x.is_ascii_alphabetic() => Some(x),
            x if x.is_ascii_whitespace() | x.is_ascii_punctuation() => Some('_'),
            _ => None,
        })
        .collect()
}
