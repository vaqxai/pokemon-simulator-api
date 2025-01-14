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

/// Endpoint for fetching a single Pokemon by its ID.
#[get("/pokemons/<name>")]
pub async fn get_pokemon<'a>(name: String) -> JsonResult<'a> {
    info!("Request to /api/pokemons/{}", name);
    let pokemon = Pokemon::get_first(&name).await.map_err(JsonStatus::from_anyhow)?;
    Ok(JsonStatus::data_owned(pokemon))
}

/// Endpoint to add a pokemon
#[post("/pokemons", data = "<pokemon>")]
pub async fn add_pokemon<'a>(mut pokemon: Json<Pokemon>) -> JsonResult<'a> {
    info!("Request to /api/pokemons");

    if pokemon.name.len() > 30 {
        return Err(JsonStatus::error("Name is too long"));
    }

    if pokemon.name.is_empty() {
        return Err(JsonStatus::error("Name cannot be empty"));
    }

    // remove slashes because of GET incompatiblity
    pokemon.name = pokemon.name.replace("\\", "");

    // do not allow duplicates
    if Pokemon::get_first(&pokemon.name).await.is_ok() {
        return Err(JsonStatus::error("Pokemon already exists"));
    }

    let mut pokemon = pokemon.into_inner();
    pokemon
        .put_with_relationships()
        .await
        .map_err(JsonStatus::from_anyhow)?;

    Ok(JsonStatus::new_empty(json::Status::Ok))
}
