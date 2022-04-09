use std::fs::File;

use common::types::RecipeBook;
use juniper::{EmptyMutation, EmptySubscription, RootNode};
use rocket::Rocket;

use crate::schema::{Context, Query};
use crate::graphql_endpoints::{graphiql, get_graphql_handler, post_graphql_handler};

mod schema;
mod graphql_endpoints;
mod merge_ingredients_endpoints;

type Schema = RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref RECIPE_BOOK: RecipeBook = {
        let reader =
            &File::open("data/RecipeBook.json").expect("Failed to find data/RecipeBook.json");

        serde_json::from_reader(reader).unwrap()
    };
}

#[rocket::main]
async fn main() {
    Rocket::build()
        .manage(Context::new())
        .manage(Schema::new(
            Query,
            EmptyMutation::<Context>::new(),
            EmptySubscription::<Context>::new(),
        ))
        .mount(
            "/",
            rocket::routes![graphiql, get_graphql_handler, post_graphql_handler],
        )
        .launch()
        .await
        .expect("server to launch");
}
