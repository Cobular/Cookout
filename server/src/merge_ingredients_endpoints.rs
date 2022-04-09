use rocket::response::content;

struct OptimizeInput {
  recipes: Vec<Recipe>,
}

#[rocket::post("/optimize", data = "<")]
pub fn graphiql() -> content::Html<String> {
  
}