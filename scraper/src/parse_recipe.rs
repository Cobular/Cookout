use std::io::{Error, ErrorKind};

use anyhow::Context;
use scraper::Html;

use crate::{NAME_SELECTOR, INGREDIENTS_SELECTOR, INSTRUCTIONS_TEXT_SELECTOR};

use common::types::{Ingredient, Recipe, Instruction};

fn parse_instructions(content: &str) -> Vec<Instruction> {
  let parsed_html = Html::parse_document(content);

  let instructions = parsed_html
      .select(&INSTRUCTIONS_TEXT_SELECTOR)
      .flat_map(|el| el.text())
      .map(|el_text: &str| Instruction { instruction: el_text.to_string() })
      .collect::<Vec<Instruction>>();

  instructions
}

fn parse_ingredients(content: &str) -> Vec<Ingredient> {
  let parsed_html = Html::parse_document(content);

  let mut ingredients: Vec<Ingredient> = Vec::new();

  for raw_ingredient in parsed_html
      .select(&INGREDIENTS_SELECTOR)
      .flat_map(|el| el.text())
      .collect::<Vec<&str>>()
      .as_slice()
      .chunks_exact(2)
  {
      ingredients.push(Ingredient {
          quantity: raw_ingredient[0].to_owned(),
          name: raw_ingredient[1].to_owned(),
      })
  }

  ingredients
}

fn parse_name(content: &str) -> Result<String, Error> {
  let parsed_html = Html::parse_document(content);

  let name_element = match parsed_html.select(&NAME_SELECTOR).next() {
      Some(name_element) => name_element.inner_html(),
      None => {
          return Err(Error::new(
              ErrorKind::Other,
              "Failed to find name of recipe",
          ))
      }
  };

  Ok(name_element)
}

pub fn parse_recipe(hellofresh_page_content: &str, url: &str) -> anyhow::Result<Recipe> {
  let url = url.to_owned();
  let name = parse_name(hellofresh_page_content).context(url.to_owned())?;
  Ok(Recipe {
      name,
      url,
      ingredients: parse_ingredients(hellofresh_page_content),
      instructions: parse_instructions(hellofresh_page_content),
  })
}