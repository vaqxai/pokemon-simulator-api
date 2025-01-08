//! A Pokemon battle simulator API server.
//!
//! This crate provides a web server implementation for simulating Pokemon battles.
//! It uses Rocket framework for handling HTTP requests and implements CORS support
//! for cross-origin requests.
//!
//! # Features
//!
//! - RESTful API endpoints for Pokemon battle simulation
//! - CORS support for cross-origin requests
//! - JSON response formatting
//!
//! # API Endpoints
//!
//! - `GET /api/` - Health check endpoint that returns OK status
#![feature(associated_type_defaults)]
#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]

use std::str::FromStr;

/// Module containing JSON-related types and functionality for API responses.
pub mod json;

/// Module containing Pokemon-related types.
pub mod pokemon;

/// Module containing Trainer-related types.
pub mod trainer;

/// Module defining basic database traits and operations
pub mod database;

#[doc(hidden)]
mod tests;
use crate::json::JsonResult;
use database::get::DbGet;
use json::JsonStatus;
use log::info;
use pokemon::Pokemon;
use rocket::serde::json::Json;
use rocket_cors::{AllowedMethods, AllowedOrigins, CorsOptions};

#[macro_use]
extern crate rocket;

/// Creates a CORS fairing with the specified configuration.
/// Allows all origins, GET, POST, and DELETE methods, and credentials.
/// # Returns
/// A `Cors` fairing with the specified configuration.
/// # Examples
/// ```
/// let cors = make_cors();
/// ```
fn make_cors() -> CorsOptions {
    let allowed_methods: AllowedMethods = ["Get", "Post", "Delete"]
        .iter()
        .map(|s| FromStr::from_str(s).unwrap())
        .collect();

    CorsOptions::default()
        // or use .allowed_origins(AllowedOrigins::some_exact(&["http://localhost:3000"])) for more restriction
        // for react frontend
        // TODO: Only allow localhost requests
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(allowed_methods)
        .allow_credentials(true)
}

#[launch]
#[tokio::main]
async fn rocket() -> _ {
    env_logger::init();
    let cors = make_cors().to_cors().expect("Error creating CORS fairing");

    rocket::build()
        .attach(cors)
        .mount("/api", routes![index, get_pokemons, add_pokemon])
}

/// Health check endpoint that returns an OK status.
/// # Returns
/// A JSON response with an OK status and no data.
/// # Examples
/// ```
/// use rocket::local::blocking::Client;
/// let client = Client::tracked(create_test_rocket()).expect("Failed to create client");
/// let response = client.get("/api").dispatch();
/// assert_eq!(response.status(), Status::Ok);
/// ```
/// # Errors
/// If the request fails, an error status is returned.
#[get("/")]
pub async fn index<'a>() -> JsonResult<'a> {
    info!("Request to /api");
    Ok(JsonStatus::ok::<String>(None))
}

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
