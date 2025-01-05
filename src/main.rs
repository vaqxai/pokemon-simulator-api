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
#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]

use std::str::FromStr;

/// Module containing JSON-related types and functionality for API responses.
pub mod json;

#[doc(hidden)]
mod tests;
use crate::json::JsonResult;
use json::JsonStatus;
use log::info;
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
    let cors = make_cors().to_cors().expect("Error creating CORS fairing");

    rocket::build().attach(cors).mount("/api", routes![index])
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
