use std::{env, fs::File};

use juniper::{EmptyMutation, EmptySubscription};

use common::types::RecipeBook;
use warp::{hyper::Response, Filter};

use crate::query::{Query, Context};

mod query;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref RECIPE_BOOK: RecipeBook = {
        let reader =
            &File::open("data/RecipeBook.json").expect("Failed to find data/RecipeBook.json");

        serde_json::from_reader(reader).unwrap()
    };
}

// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
type Schema = juniper::RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;

fn schema() -> Schema {
    Schema::new(
        Query,
        EmptyMutation::<Context>::new(),
        EmptySubscription::<Context>::new(),
    )
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "warp_server");
    env_logger::init();

    let log = warp::log("warp_server");

    let homepage = warp::path::end().map(|| {
        Response::builder()
            .header("content-type", "text/html")
            .body(
                "<html><h1>juniper_warp</h1><div>visit <a href=\"/graphiql\">/graphiql</a></html>"
                    .to_string(),
            )
    });

    println!("Listening on 127.0.0.1:8080");

    let state = warp::any().map(|| {
        Context {
            database: &RECIPE_BOOK
        }
    });
    let graphql_filter = juniper_warp::make_graphql_filter(schema(), state.boxed());

    warp::serve(
        warp::get()
            .and(warp::path("graphiql"))
            .and(juniper_warp::graphiql_filter("/graphql", None))
            .or(homepage)
            .or(warp::path("graphql").and(graphql_filter))
            .with(log),
    )
    .run(([127, 0, 0, 1], 8080))
    .await
}
