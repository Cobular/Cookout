use std::collections::HashSet;

use futures::{stream, StreamExt};
use reqwest::{Client, Response};
use scraper::Html;

use crate::{utils::load_page, LINK_SELECTOR, RECIPE_URL_REGEX};

fn parse_main(content: &str) -> Vec<String> {
    let parsed_html = Html::parse_document(content);

    parsed_html
        .select(&LINK_SELECTOR)
        .flat_map(|el| el.value().attr("href"))
        .filter(|url| RECIPE_URL_REGEX.is_match(url))
        .map(String::from)
        .collect::<Vec<String>>()
}

pub async fn parse_main_from_url(client: &Client, urls: Vec<&str>) -> reqwest::Result<Vec<String>> {
    let mut url_set: HashSet<String> = HashSet::new();

    for url in urls {
        let res = load_page(client, url).await?.text().await?;
        url_set.extend(parse_main(&res));
    }

    Ok(url_set.into_iter().collect())
}

pub async fn parse_main_from_url_fast(
    client: &Client,
    urls: Vec<&str>,
) -> reqwest::Result<Vec<String>> {
    let mut url_set: HashSet<String> = HashSet::new();

    let recipe_pages = stream::iter(urls)
        .map(|url| async move { load_page(client, url).await })
        .buffer_unordered(30);

    let thing: Vec<Vec<String>> = recipe_pages
        .filter_map(|recipe_page: reqwest::Result<Response>| async {
            if let Ok(recipe_page) = recipe_page {
                if let Ok(text) = recipe_page.text().await {
                    Some(parse_main(&text))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<Vec<String>>>()
        .await;

    for recipe in thing {
        url_set.extend(recipe);
    }

    Ok(url_set.into_iter().collect())
}
