use std::{error::Error, fs::File, time::SystemTime, vec};

use futures::{stream, StreamExt};
use regex::Regex;
use reqwest::{Client, Response};
use scraper::Selector;

use common::types::{Recipe, RecipeBook};

use crate::utils::{load_page, prep_name_for_file};

mod parse_main;
mod parse_recipe;
mod utils;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref INSTRUCTIONS_TEXT_SELECTOR: Selector =
        Selector::parse("div[data-test-id^=recipeDetailFragment\\.instructions\\.step] p")
            .expect("Error during the parsing of the instructions text selector");
    static ref INGREDIENTS_SELECTOR: Selector = {
        let paragraph_selector = "div[data-test-id=recipeDetailFragment\\.ingredients] p";
        Selector::parse(&format!("{paragraph_selector}:nth-child(1):nth-last-child(2), {paragraph_selector}:nth-child(2):nth-last-child(1)"))
            .expect("Error during the parsing of the ingredient text selector")
    };
    static ref NAME_SELECTOR: Selector =
        Selector::parse("h1[data-test-id=recipeDetailFragment\\.recipe-name]")
            .expect("Error during the parsing of the recipe name selector");
    static ref LINK_SELECTOR: Selector =
        Selector::parse("a[href^=https\\:\\/\\/www\\.hellofresh\\.com\\/recipes\\/]")
            .expect("Error during parsing of the recipe link selector");
    static ref RECIPE_URL_REGEX: Regex =
        Regex::new(r"https://www\.hellofresh\.com/recipes/.*-[a-f0-9]{24}")
            .expect("Error during parsing of recipe url regex");
}

async fn parse_from_url(client: &Client, url: &str) -> anyhow::Result<Recipe> {
    let res = load_page(client, url).await?.text().await?;
    Ok(parse_recipe::parse_recipe(&res, url)?)
}

async fn parse_from_url_to_file(
    client: &Client,
    url: &str,
    folder: Option<&str>,
) -> Result<Recipe, Box<dyn Error>> {
    let recipe = parse_from_url(client, url).await?;
    serde_json::to_writer(
        &File::create(&format!(
            "{}/{}.json",
            folder.unwrap_or("."),
            prep_name_for_file(&recipe.name)
        ))?,
        &recipe,
    )?;
    Ok(recipe)
}

async fn parse_recipe_book(client: &Client, urls: Vec<&str>) -> anyhow::Result<RecipeBook> {
    let mut recipe_book = RecipeBook::new();
    let map = &mut recipe_book.0;

    let recipe_pages = stream::iter(urls)
        .map(|url| async move { load_page(client, url).await })
        .buffer_unordered(50);

    let thing: Vec<anyhow::Result<Recipe>> = recipe_pages.filter_map(|recipe_page: reqwest::Result<Response>| async {
        if let Ok(recipe_page) = recipe_page {
            let url = recipe_page.url().to_owned();
            if let Ok(text) = recipe_page.text().await {
                Some(parse_recipe::parse_recipe(&text, url.as_str()))
            } else {
                None
            }
        } else {
            None
        }
    }).collect::<Vec<anyhow::Result<Recipe>>>().await;

    for recipe in thing.into_iter().flatten() {
        map.insert(prep_name_for_file(recipe.name.as_str()), recipe);
    }

    Ok(recipe_book)
}

#[tokio::main]
async fn main() {
    let client = Client::builder().cookie_store(true).build().unwrap();

    let res = parse_main::parse_main_from_url_fast(
        &client,
        vec![
            "https://www.hellofresh.com/recipes",
            "https://www.hellofresh.com/recipes/american-recipes",
            "https://www.hellofresh.com/recipes/asian-recipes",
            "https://www.hellofresh.com/recipes/italian-recipes",
            "https://www.hellofresh.com/recipes/mediterranean-recipes",
            "https://www.hellofresh.com/recipes/korean-recipes",
            "https://www.hellofresh.com/recipes/indian-recipes",
            "https://www.hellofresh.com/recipes/latin-american-recipes",
            "https://www.hellofresh.com/recipes/chinese-recipes",
            "https://www.hellofresh.com/recipes/spanish-recipes",
            "https://www.hellofresh.com/recipes/japanese-recipes",
            "https://www.hellofresh.com/recipes/thai-recipes",
            "https://www.hellofresh.com/recipes/french-recipes",
            "https://www.hellofresh.com/recipes/cuban-recipes",
            "https://www.hellofresh.com/recipes/african-recipes",
            "https://www.hellofresh.com/recipes/cajun-recipes",
            "https://www.hellofresh.com/recipes/middle-eastern-recipes",
            "https://www.hellofresh.com/recipes/vietnamese-recipes",
            "https://www.hellofresh.com/recipes/hawaiian-recipes",
            "https://www.hellofresh.com/recipes/taco-recipes",
            "https://www.hellofresh.com/recipes/burger-recipes",
            "https://www.hellofresh.com/recipes/pasta-recipes",
            "https://www.hellofresh.com/recipes/bowl-recipes",
            "https://www.hellofresh.com/recipes/flatbread-recipes",
            "https://www.hellofresh.com/recipes/stir-fry-recipes",
            "https://www.hellofresh.com/recipes/meatball-recipes",
            "https://www.hellofresh.com/recipes/noodle-recipes",
            "https://www.hellofresh.com/recipes/risotto-recipes",
            "https://www.hellofresh.com/recipes/skillet-recipes",
            "https://www.hellofresh.com/recipes/soup-recipes",
            "https://www.hellofresh.com/recipes/skewer-recipes",
            "https://www.hellofresh.com/recipes/quesadilla-recipes",
            "https://www.hellofresh.com/recipes/meatloaf-recipes",
            "https://www.hellofresh.com/recipes/fajita-recipes",
            "https://www.hellofresh.com/recipes/enchilada-recipes",
            "https://www.hellofresh.com/recipes/bibimbap-recipes",
            "https://www.hellofresh.com/recipes/burrito-recipes",
            "https://www.hellofresh.com/recipes/sandwich-recipes",
            "https://www.hellofresh.com/recipes/tostada-recipes",
            "https://www.hellofresh.com/recipes/casserole-recipes",
            "https://www.hellofresh.com/recipes/quick-meals",
            "https://www.hellofresh.com/recipes/mexican-recipes",
            "https://www.hellofresh.com/recipes/pasta-recipes",
            "https://www.hellofresh.com/recipes/easy-recipes",
            "https://www.hellofresh.com/recipes/mexican-recipes/tacos",
            "https://www.hellofresh.com/recipes/kid-friendly-recipes",
            "https://www.hellofresh.com/recipes/hall-of-fame",
        ],
    )
    .await
    .unwrap();

    println!("Number URLs found: {:#?}", res.len());

    let res = parse_recipe_book(&client, res.iter().map(|s| s as &str).collect())
        .await
        .unwrap();

    let time_string: String = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs().to_string(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    serde_json::to_writer(
        &File::create(&format!("{}/RecipeBook-{}.json", "data", time_string)).unwrap(),
        &res,
    )
    .unwrap();
}
