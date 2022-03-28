#[macro_use] extern crate juniper;

use juniper::{FieldResult};

use common::types::{RecipeBook};

struct Context {
    // Use your real database pool here.
    database: RecipeBook,
}

// To make our context usable by Juniper, we have to implement a marker trait.
impl juniper::Context for Context {}

fn main() {
    println!("Hello, world!");
}
