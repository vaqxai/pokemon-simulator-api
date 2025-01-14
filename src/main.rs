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

use std::{env, net::Ipv4Addr, str::FromStr};

/// Module containing JSON-related types and functionality for API responses.
pub mod json;

/// Module containing Pokemon-related types.
pub mod pokemon;

/// Module containing Trainer-related types.
pub mod trainer;

/// Module defining basic database traits and operations
pub mod database;

/// Module containing fight simulation logic
pub mod fight;

#[doc(hidden)]
mod tests;
use crate::json::JsonResult;
use json::JsonStatus;
use log::{warn, info};
use rocket_cors::{AllowedMethods, AllowedOrigins, CorsOptions};

#[macro_use]
extern crate rocket;

const DEFAULT_CONFIG_FILE: &str = "\
[database]
";

const DEFAULT_DB_HOST: &str = "neo4j";
const DEFAULT_DB_PORT: &str = "7687";
const DEFAULT_DB_USER: &str = "neo4j";
const DEFAULT_DB_PASS: &str = "neo4j_pa$$w0rd";

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

    let args = env::args().collect::<Vec<_>>();

    if !args.is_empty() {
        info!("Running as {}", args[0].to_string());
        info!("Working directory is {}", std::env::current_dir().as_ref().unwrap().to_str().unwrap());
    };

    let db_host: &str = if args.len() > 1 && args[1].len() > 8 && args[1][..8] == *"DB_HOST=" {
        info!("Setting configured database host to: {}", args[1][8..].to_string());
        &args[1][8..]
    } else {
        DEFAULT_DB_HOST
    };

    let db_port: &str = if args.len() > 2 && args[2].len() > 8 && args[2][..8] == *"DB_PORT=" {
        info!("Setting configured database port to: {}", args[2][8..].to_string());
        &args[2][8..]
    } else {
        DEFAULT_DB_PORT
    };

    let db_user: &str = if args.len() > 3 && args[3].len() > 8 && args[3][..8] == *"DB_USER=" {
        info!("Setting configured database user to: {}", args[3][8..].to_string());
        &args[3][8..]
    } else {
        DEFAULT_DB_USER
    };

    let db_pass: &str = if args.len() > 4 && args[4].len() > 8 && args[4][..8] == *"DB_PASS=" {
        info!("Setting configured database password to: {}", args[4][8..].to_string());
        &args[4][8..]
    } else {
        DEFAULT_DB_PASS
    };

    // Create default config file for the database, if it doesn't exist
    if std::fs::exists("config/config.toml").is_ok_and(|e| !e) {
        let mut def_cfg = DEFAULT_CONFIG_FILE.to_string();

        def_cfg = def_cfg + "host = \"" + db_host + "\"\n";
        def_cfg = def_cfg + "port = \"" + db_port + "\"\n";
        def_cfg = def_cfg + "username = \"" + db_user + "\"\n";
        def_cfg = def_cfg + "password = \"" + db_pass + "\"\n";
        
        info!("Generated config file:\n{}", def_cfg);

        if let Err(e) = std::fs::write(
            "config/config.toml",
            def_cfg.as_bytes()
        ) {
            warn!("The config file could not be created, but the program will continue anyway.");
            warn!("This could cause further issues.");
            warn!("Error writing default config file \"config/config.toml\": {e}");
        };
    } else {
        warn!("Config file already exists. Not overwriting.");
    }

    let config = rocket::Config {
        port: 8000,
        address: Ipv4Addr::new(0, 0, 0, 0).into(),
        ..rocket::Config::default()
    };

    rocket::build()
        .configure(config)
        .attach(cors)
        .mount("/api", routes![
            index,
            pokemon::endpoints::get_pokemons,
            pokemon::endpoints::add_pokemon,
            trainer::endpoints::get_trainers,
            trainer::endpoints::create_trainer,
            trainer::endpoints::delete_trainer,
            trainer::endpoints::get_trainer_pokemons,
            trainer::endpoints::add_pokemon_to_trainer,
            trainer::endpoints::remove_pokemon_from_trainer,
            fight::endpoints::simulate_fight,
            fight::endpoints::simulate_trainer_fight
        ])
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
