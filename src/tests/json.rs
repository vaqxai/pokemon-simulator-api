use erased_serde::{Serialize, serialize_trait_object};

#[allow(unused_imports)]
use crate::json::{JsonStatus, Status};

// Helper struct for testing
#[derive(serde::Serialize)]
#[allow(unused)]
struct TestData {
    field1: String,
    field2: i32,
}

// Helper struct for testing
// Define a trait for test data
#[allow(unused)]
trait TestTrait: Serialize {
    fn get_field1(&self) -> &str;
    fn get_field2(&self) -> i32;
}

serialize_trait_object!(TestTrait);

#[test]
fn test_new_owned() {
    let test_data = TestData {
        field1: "test".to_string(),
        field2: 42,
    };
    let status = JsonStatus::new_owned(Status::Ok, test_data);
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("\"status\":\"Ok\""));
    assert!(json.contains("\"field1\":\"test\""));
    assert!(json.contains("\"field2\":42"));
}

#[test]
fn test_new_empty() {
    let status = JsonStatus::new_empty(Status::Ok);
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("\"status\":\"Ok\""));
    assert!(json.contains("\"data\":[]"));
}

#[test]
fn test_error() {
    let error_msg = "Something went wrong";
    let status = JsonStatus::error(error_msg);
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("\"status\":{\"Error\":\"Something went wrong\"}"));
    assert!(json.contains("\"data\":[]"));
}

#[test]
fn test_ok_with_message() {
    let msg = "Operation successful";
    let status = JsonStatus::ok(Some(msg));
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("\"status\":\"Ok\""));
    // The message is stored as bytes, so we need to check the actual bytes
    let expected_bytes = msg.as_bytes();
    assert!(json.contains(&format!("\"data\":[{}]",
            expected_bytes.iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(","))));
}

#[test]
fn test_ok_without_message() {
    let status: JsonStatus = JsonStatus::ok(None::<String>);
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("\"status\":\"Ok\""));
    assert!(json.contains("\"data\":[]"));
}

#[test]
fn test_data_owned() {
    let test_data = TestData {
        field1: "owned data".to_string(),
        field2: 100,
    };
    let status = JsonStatus::data_owned(test_data);
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("\"status\":\"Ok\""));
    assert!(json.contains("\"field1\":\"owned data\""));
    assert!(json.contains("\"field2\":100"));
}

#[test]
fn test_from_std_error() {
    let error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let status: JsonStatus = error.into();
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("\"status\":{\"Error\":\"File not found\"}"));
}

#[test]
fn test_from_anyhow_error() {
    let error = anyhow::anyhow!("Custom error message");
    let status = JsonStatus::from_anyhow(error);
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("\"status\":{\"Error\":"));
    assert!(json.contains("Custom error message"));
}

#[test]
fn test_new_ref() {
    let test_data = TestData {
        field1: "reference data".to_string(),
        field2: 200,
    };
    let status = JsonStatus::new_ref(Status::Ok, &test_data);
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("\"status\":\"Ok\""));
    assert!(json.contains("\"field1\":\"reference data\""));
    assert!(json.contains("\"field2\":200"));
}

#[test]
fn test_data_ref() {
    let test_data = TestData {
        field1: "another reference".to_string(),
        field2: 300,
    };
    let status = JsonStatus::data_ref(&test_data);
    let json = serde_json::to_string(&status).unwrap();
    assert!(json.contains("\"status\":\"Ok\""));
    assert!(json.contains("\"field1\":\"another reference\""));
    assert!(json.contains("\"field2\":300"));
}

#[test]
fn test_into_raw_json() {
    use rocket::response::content::RawJson;

    let test_data = TestData {
        field1: "raw json test".to_string(),
        field2: 400,
    };
    let status = JsonStatus::data_owned(test_data);
    let raw_json: RawJson<String> = status.into();
    let json_str = raw_json.0;
    assert!(json_str.contains("\"status\":\"Ok\""));
    assert!(json_str.contains("\"field1\":\"raw json test\""));
    assert!(json_str.contains("\"field2\":400"));
}
