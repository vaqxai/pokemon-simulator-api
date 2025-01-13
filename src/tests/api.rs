use super::super::*;
use rocket::Build;
use rocket::Rocket;
#[allow(unused_imports)]
use rocket::http::{Header, Status};
#[allow(unused_imports)]
use rocket::local::blocking::Client;

#[allow(unused)]
fn create_test_rocket() -> Rocket<Build> {
    let cors = make_cors().to_cors().expect("Error creating CORS fairing");
    rocket::build().attach(cors).mount("/api", routes![index])
}

#[test]
fn test_index_endpoint() {
    let client = Client::tracked(create_test_rocket()).expect("Failed to create client");
    let response = client.get("/api").dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body = response
        .into_string()
        .expect("Response body should be readable");
    let json: serde_json::Value =
        serde_json::from_str(&body).expect("Response should be valid JSON");

    assert_eq!(json["status"], "Ok");
    assert_eq!(json["data"], serde_json::Value::Array(vec![]));
}

#[test]
fn test_cors_headers() {
    let client = Client::tracked(create_test_rocket()).expect("Failed to create client");

    // Test simple (non-preflight) CORS request
    let response = client
        .get("/api")
        .header(Header::new("Origin", "http://localhost:3000"))
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    // Check basic CORS headers that should be present in all CORS responses
    let headers = response.headers();
    assert!(
        headers.contains("Access-Control-Allow-Origin"),
        "Response should contain Access-Control-Allow-Origin header"
    );
    assert!(
        headers.contains("Access-Control-Allow-Credentials"),
        "Response should contain Access-Control-Allow-Credentials header"
    );

    // For simple requests, Access-Control-Allow-Methods should not be present
    assert!(
        !headers.contains("Access-Control-Allow-Methods"),
        "Simple requests should not include Access-Control-Allow-Methods header"
    );

    // Test preflight request where allowed methods should be present
    let preflight_response = client
        .options("/api")
        .header(Header::new("Origin", "http://localhost:3000"))
        .header(Header::new("Access-Control-Request-Method", "GET"))
        .dispatch();

    let preflight_headers = preflight_response.headers();
    assert!(
        preflight_headers.contains("Access-Control-Allow-Methods"),
        "Preflight response should include Access-Control-Allow-Methods header"
    );

    let allowed_methods = preflight_headers
        .get_one("Access-Control-Allow-Methods")
        .expect("Should have allowed methods header in preflight response");
    assert!(allowed_methods.contains("GET"));
    assert!(allowed_methods.contains("POST"));
    assert!(allowed_methods.contains("DELETE"));
}

#[test]
fn test_preflight_request() {
    let client = Client::tracked(create_test_rocket()).expect("Failed to create client");
    let response = client
        .options("/api")
        .header(Header::new("Origin", "http://localhost:3000"))
        .header(Header::new("Access-Control-Request-Method", "GET"))
        .dispatch();

    // Both 200 OK and 204 No Content are valid for preflight responses
    let status = response.status();
    assert!(
        status == Status::Ok || status == Status::NoContent,
        "Expected status 200 OK or 204 No Content, got {}",
        status
    );

    // Check preflight specific headers
    let headers = response.headers();
    assert!(headers.contains("Access-Control-Allow-Origin"));
    assert!(headers.contains("Access-Control-Allow-Methods"));
    assert!(headers.contains("Access-Control-Allow-Credentials"));

    // Additional preflight response validation
    let allowed_methods = headers
        .get_one("Access-Control-Allow-Methods")
        .expect("Should have allowed methods header");
    assert!(allowed_methods.contains("GET"));

    // Validate max age if present
    if let Some(max_age) = headers.get_one("Access-Control-Max-Age") {
        assert!(
            max_age.parse::<i32>().is_ok(),
            "Max-Age should be a valid integer"
        );
    }
}

#[test]
fn test_invalid_method() {
    let client = Client::tracked(create_test_rocket()).expect("Failed to create client");
    let response = client.put("/api").dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

#[test]
fn test_nonexistent_endpoint() {
    let client = Client::tracked(create_test_rocket()).expect("Failed to create client");
    let response = client.get("/api/nonexistent").dispatch();

    assert_eq!(response.status(), Status::NotFound);
}

// Integration tests for JSON response structure
#[test]
fn test_json_response_structure() {
    let client = Client::tracked(create_test_rocket()).expect("Failed to create client");
    let response = client.get("/api").dispatch();

    assert_eq!(response.status(), Status::Ok);

    let body = response
        .into_string()
        .expect("Response body should be readable");
    let json: serde_json::Value =
        serde_json::from_str(&body).expect("Response should be valid JSON");

    // Check JSON structure
    assert!(json.is_object());
    assert!(json.get("status").is_some());
    assert!(json.get("data").is_some());

    // Verify types
    assert!(json["status"].is_string());
    assert_eq!(json["status"], String::from("Ok"));
}

// Test helper functions
#[test]
fn test_cors_configuration() {
    let cors = make_cors();

    // Verify CORS settings
    assert!(cors.allow_credentials);
    assert!(matches!(cors.allowed_origins, AllowedOrigins::All));

    // Verify allowed methods
    let methods: Vec<_> = cors.allowed_methods.iter().collect();
    assert_eq!(methods.len(), 3);
    assert!(methods.iter().any(|m| m.as_str() == "GET"));
    assert!(methods.iter().any(|m| m.as_str() == "POST"));
    assert!(methods.iter().any(|m| m.as_str() == "DELETE"));
}
