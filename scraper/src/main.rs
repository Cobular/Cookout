use std::{error::Error, fs::File, time::SystemTime};

use log::error;
use reqwest::blocking::Client;
use scraper::Selector;

use common::types::{Recipe, RecipeBook};

use crate::utils::{load_page, prep_name_for_file};

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
}

fn parse_from_url(client: &Client, url: &str) -> reqwest::Result<Recipe> {
    let res = load_page(client, url)?.text()?;
    Ok(parse_recipe::parse_recipe(&res, url))
}

fn parse_from_url_to_file(
    client: &Client,
    url: &str,
    folder: Option<&str>,
) -> Result<Recipe, Box<dyn Error>> {
    let recipe = parse_from_url(client, url)?;
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

fn parse_recipe_book(client: &Client, urls: Vec<&str>) -> RecipeBook {
    let mut recipe_book = RecipeBook::new();
    let map = &mut recipe_book.0;
    for url in urls {
        match parse_from_url(client, url) {
            Ok(recipe) => (*map).insert(prep_name_for_file(&recipe.name).to_owned(), recipe),
            Err(err) => {
                error!("{}", err);
                continue;
            }
        };
    }
    recipe_book
}

fn main() {
    let client = Client::builder().cookie_store(true).build().unwrap();

    let res = parse_recipe_book(
        &client,
        vec![
            "https://www.hellofresh.com/recipes/crispy-chipotle-shrimp-tacos-5a7a2e4eae08b52c90355af2",
            "https://www.hellofresh.com/recipes/pineapple-poblano-beef-tacos-5d60394f053f38001054c40a"
        ]
    );

    let time_string: String = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs().to_string(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    serde_json::to_writer(
        &File::create(&format!(
            "{}/RecipeBook-{}.json",
            "data",
            time_string
        )).unwrap(),
        &res,
    ).unwrap();
}
