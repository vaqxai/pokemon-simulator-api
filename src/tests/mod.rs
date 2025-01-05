mod api;
mod json;

/// Test if rocket can be built
#[test]
fn test_rocket() {
    use crate::rocket;

    let _rocket = rocket();
    // no panic = success
}
