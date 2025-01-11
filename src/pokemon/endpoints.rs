use rocket::serde::json::Json;

use crate::{
    database::get::DbGet,
    json::{self, JsonResult, JsonStatus},
    pokemon::Pokemon,
};

/// Endpoint for getting a list of all Pokemon.
#[get("/pokemons")]
pub async fn get_pokemons<'a>() -> JsonResult<'a> {
    info!("Request to /api/pokemons");
    let pokemons = Pokemon::get_all().await.map_err(JsonStatus::from_anyhow)?;
    Ok(JsonStatus::data_owned(pokemons))
}

/// Endpoint to add a pokemon
#[post("/pokemons", data = "<pokemon>")]
pub async fn add_pokemon<'a>(pokemon: Json<Pokemon>) -> JsonResult<'a> {
    info!("Request to /api/pokemons");

    if pokemon.name.len() > 30 {
        return Err(JsonStatus::error("Name is too long"));
    }

    if pokemon.name.is_empty() {
        return Err(JsonStatus::error("Name cannot be empty"));
    }

    let mut pokemon = pokemon.into_inner();
    pokemon
        .put_with_relationships()
        .await
        .map_err(JsonStatus::from_anyhow)?;

    Ok(JsonStatus::new_empty(json::Status::Ok))
}
