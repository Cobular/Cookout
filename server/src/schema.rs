use common::types::{RecipeBook, Recipe};
use juniper::{graphql_object, FieldResult};

use crate::RECIPE_BOOK;


pub struct Context {
  pub database: &'static RecipeBook,
}

// To make our context usable by Juniper, we have to implement a marker trait.
impl juniper::Context for Context {}

impl Context {
  pub fn new() -> Self {
    Context {
      database: &RECIPE_BOOK,
    }
  }
}

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    fn apiVersion() -> &'static str {
        "1.0"
    }

    // Arguments to resolvers can either be simple types or input objects.
    // To gain access to the context, we specify a argument
    // that is a reference to the Context type.
    // Juniper automatically injects the correct context here.
    fn recipe(context: &Context, key: String) -> FieldResult<&Recipe> {
        let map = &context.database.0;

        #[allow(clippy::try_err)]
        let recipe = match map.get(&key) {
            Some(recipe) => recipe,
            _ => Err("Could not find recipe with key")?,
        };

        // Return the result.
        Ok(recipe)
    }

    /// Get recipes with pagination
    fn recipes(context: &Context, start: i32, count: i32) -> FieldResult<Vec<Recipe>> {
        let map = &context.database.0;

        let mut values = map.values();

        // Start the pagination. Need to skip 0 and do one less for all greater numbers
        if start < 0 || count < 0 {
            #[allow(clippy::try_err)]
            Err("Must provide positive start and count")?;
        }
        if start >= 1 && values.nth((start - 1).try_into()?).is_none() {
            #[allow(clippy::try_err)]
            Err("Tried to start off the end of the set of recipes")?
        }

        // Return the result.
        Ok(values
            .take(count.try_into()?)
            .cloned()
            .collect::<Vec<Recipe>>())
    }

    /// List all recipe names
    fn recipe_names(context: &Context) -> FieldResult<Vec<String>> {
        let map = &context.database.0;

        let values = map.keys();

        // Return the result.
        Ok(values.cloned().collect())
    }
}
