use std::{env, fs::File};

use juniper::{graphql_object, EmptyMutation, EmptySubscription, FieldResult};

use common::types::{Recipe, RecipeBook};
use warp::{hyper::Response, Filter};

struct Context {
    database: &'static RecipeBook,
}

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref RECIPE_BOOK: RecipeBook = {
        let reader =
            &File::open("data/RecipeBook.json").expect("Failed to find data/RecipeBook.json");

        serde_json::from_reader(reader).unwrap()
    };
}

// To make our context usable by Juniper, we have to implement a marker trait.
impl juniper::Context for Context {}

struct Query;

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
