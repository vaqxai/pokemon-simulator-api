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
//! # Pokemon Simulator
//! 
//! A Pokemon battle simulator API server.
//! 
//! This crate provides a web server implementation for simulating Pokemon battles.
//! It uses Rocket framework for handling HTTP requests and implements CORS support
//! for cross-origin requests.
//! 
//! ## Features
//! 
//! - RESTful API endpoints for Pokemon battle simulation
//! - CORS support for cross-origin requests
//! - JSON response formatting
//! 
//! ## API Endpoints
//! 
//! - `GET /api/` - Health check endpoint that returns OK status
//! - `GET /api/pokemons` - A list of all pokemons
//! - `POST /api/pokemons` - With a pokemon JSON in the body (same format as what comes from the `GET /api/pokemons` endpoint) adds a new pokemon
//! - `GET /api/trainers` - A list of all trainers and their pokemon
//! - `POST /api/trainer_pokemons/<trainer_name>` - Creates a trainer with name `<trainer_name>`
//! - `DELETE /api/trainer_pokemons/<trainer_name>` - Deletes a trainer with name `<trainer_name>`
//! - `GET /api/trainer_pokemons/<trainer_name>` - Returns a full list of all pokemons of a particular trainer
//! - `POST /api/trainer_pokemons/<trainer_name>/<pokemon_name>` - Adds a pokemon to a trainer's team, name is case sensitive.
//! - `DELETE /api/trainer_pokemons/<trainer_name>/<pokemon_name>` - Removes a pokemon from a trainer's team, name is case sensitive.
//! - `GET /api/simulate_fight/<contender_name>/<challenger_name>` - Simulate a fight between two pokemon, names are case sensitive.
//! - `GET /api/simulate_trainer_fight/<challenger_name>/<challenger_strategy>/<contender_name>/<contender_strategy>` - Simulate a fight between two trainers, possible strategies listed below, names are case sensitive.
//! 
//! ### Fight Strageies
//! The fight strategy changhes how a trainer picks their next pokemon upon a pokemon's faint, or the first pokemon to go and battle
//! - `StrongestAtk` - Always choose the pokemon with the highest attack stat in your team
//! - `StrongestDef` - Always choose the pokemon with the highest defense stat in your team
//! - `StrongestSum` - Always choose the pokemon that has the highest atk+def sum
//! - `StrongestType` - If you can, choose a pokemon that has the best type advantage over the current enemy pokemon, or, if not possible, use `StrongestSum` instead
//! - `Random` - Always choose a random pokemon
//! ## Pokemon Fight Algorithm
//! 1. The pokemon with the highest `AGI`lity stat attacks first
//! 2. The base damage is the pokemon's `ATK` (attack) stat
//! 3. The type damage multiplier is calculated as follows, starting with a multiplier of `1`
//!     1. If the attacker's primary type is "Strong Against" the defender's primary type, add `0.375` to the type damage multiplier
//!     2. If the attacker's primary type is "Weak Against" the defender's primary type, subtract `0.225` from the type damage multiplier
//!     3. If the defender has a secondary type, and the attacker's primary type is "Strong Against" it, add `0.375` to the type damage multiplier
//!     4. If the defender has a secondary type, and the attacker's primary type is "Weak Against" it, subtract `0.225` from the type damage multiplier
//!     5. If the attacker has a secondary type, and the defender's primary type is "Weak Against" it, add `0.375` to the type damage multiplier
//!     6. If the attacker has a secondary type, and the defender's primary type is "Strong Against" it, subtract `0.225` from the type damage multiplier
//!     7. If both pokemon have a secondary type, and the defender's is "Weak Against" the attacker's, add `0.375` to the type damage multiplier
//!     8. If both pokemon have a secondary type, and the defender's is "Strong Against" the attacker's, subtract `0.225` from the type damage multiplier
//!        
//! 4. The maximum type damage multiplier is `2.5`, the minimum is `0.1`. A type damage multiplier above `1.8` means an attack is "Super effective", while a type damage multiplier below `0.6` means an attack is "Not very effective"
//! 5. A random multiplier between `0.8` and `1.2` is calculated
//! 6. A defense multiplier is calculated by dividing the defender's `DEF`ense stat by the maximum value of `250.0`, multiplied by `0.75`, then subtracted from `1.0`, to give a total defense multiplier (which multiplies the damage incoming to the defender) between `0.0` for a `0 DEF` stat, and `0.75` for a `250 DEF` stat
//! 7. The base damage is multiplied by the type damage multiplier, the random multiplier, and the defense multiplier.
//! 8. The final damage is subtracted from the defender's `HP` (hit points) stat.
//! 9. If the defender's `HP` falls below zero, a fight is concluded.
//! 10. Otherwise, the roles of the attacker and the defender are reversed, the remaining `HP` is carried over to the next round, and the fight continues until one of the pokemons' `HP` falls to zero.
//! 
//! ## Trainer Fight Algorithm
//! 1. The trainer picked as the `contender` picks their pokemon first. If they've selected the `StrongestType` strategy, they use `StrongestSum` for their first pokemon instead (as the other party has yet to choose their pokemon)
//! 2. The trainer picked as the `challenger` picks their pokemon according to their strategy.
//! 3. The two pokemon fight using the regular Pokemon Fight Algorithm
//! 4. The winner's remaining `HP` is carried over to the next round, and the party whose pokemon fainted picks a new one using their strategy.
//! 5. The first party to run out of pokemon loses the battle.

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
const DEFAULT_DB_PASS: &str = "pass";

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
        def_cfg = def_cfg + "port = " + db_port + "\n";
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
    }warn!("Config file already exists. Not overwriting.");

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
