mod api;
mod json;

mod database;

mod pokemon;

/// Test if rocket can be built
#[test]
fn test_rocket() {
    use crate::rocket;

    let _rocket = rocket();
    // no panic = success
}
