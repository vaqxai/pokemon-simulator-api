use std::str::FromStr;

mod json;
use crate::json::JsonResult;
use json::JsonStatus;
use log::info;
use rocket_cors::{AllowedMethods, AllowedOrigins, CorsOptions};

#[macro_use]
extern crate rocket;

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

#[get("/")]
pub async fn index<'a>() -> JsonResult<'a> {
    info!("Request to /api");
    Ok(JsonStatus::ok::<String>(None))
}
