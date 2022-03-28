use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, GraphQLObject)]
#[graphql(description="An ingredient in a recipe")]
pub struct Ingredient {
    pub name: String,
    pub quantity: String,
}

#[derive(Debug, Serialize, Deserialize, GraphQLObject)]
#[graphql(description="An instruction in a recipe")]
pub struct Instruction {
    pub instruction: String,
}

#[derive(Debug, Serialize, Deserialize, GraphQLObject)]
#[graphql(description="A whole recipe entry")]
pub struct Recipe {
    pub name: String,
    pub url: String,
    pub ingredients: Vec<Ingredient>,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecipeBook(pub HashMap<String, Recipe>);

impl RecipeBook {
    pub fn new() -> RecipeBook {
        let hashmap= HashMap::new();
        RecipeBook(hashmap)
    }
}

impl Default for RecipeBook {
    fn default() -> Self {
        Self::new()
    }
}