use erased_serde::Serialize as EraSerialize;
use rocket::response::Responder;
use rocket::response::content::RawJson;
use serde::Serialize;

#[derive(Serialize)]
pub enum Status {
    Ok,
    /// Represents an error with a message describing the error.
    Error(String),
}

#[derive(Serialize)]
#[serde(untagged)]
enum Data<'a> {
    /// An enum variant that holds a boxed trait object implementing the `EraSerialize` trait.
    /// The trait object is required to have a static lifetime.
    Owned(Box<dyn EraSerialize + 'static>),
    Ref(&'a dyn EraSerialize),
}

#[derive(Serialize)]
/// A struct representing a JSON status with associated data.
///
/// # Fields
///
/// * `status` - The status of the JSON response.
/// * `data` - The data associated with the JSON response, with a lifetime `'a`.
pub struct JsonStatus<'a> {
    status: Status,
    data: Data<'a>,
}

/// Converts a `JsonStatus` into a `RawJson<String>`.
///
/// This implementation uses `serde_json` to serialize the `JsonStatus`
/// into a JSON string and then wraps it in a `RawJson`.
///
/// # Panics
///
/// This function will panic if the serialization of `JsonStatus` fails.
///
/// # Examples
///
/// ```
/// let status = JsonStatus { /* fields */ };
/// let raw_json: RawJson<String> = status.into();
/// ```
impl From<JsonStatus<'_>> for RawJson<String> {
    fn from(status: JsonStatus) -> Self {
        RawJson(serde_json::to_string(&status).unwrap())
    }
}

/// Implements the `From` trait for `JsonStatus<'static>` to allow conversion from any type that
/// implements the `std::error::Error` trait. This implementation logs the error and creates a
/// `JsonStatus` instance representing the error.
///
/// # Arguments
///
/// * `error` - An error of type `T` that implements the `std::error::Error` trait.
///
/// # Returns
///
/// * A `JsonStatus` instance representing the error.
impl<T: std::error::Error> From<T> for JsonStatus<'static> {
    fn from(error: T) -> Self {
        info!("Error while running request: {}", error);
        JsonStatus::error(format!("{}", error))
    }
}

pub type JsonResult<'a> = Result<JsonStatus<'a>, JsonStatus<'static>>;

/// Implementation of `JsonStatus` with a static lifetime.
///
/// Provides methods to create instances of `JsonStatus` with different statuses and data.
///
/// # Methods
///
/// - `new_owned`: Creates a new `JsonStatus` with the given status and owned data.
/// - `new_empty`: Creates a new `JsonStatus` with the given status and empty data.
/// - `error`: Creates a new `JsonStatus` with an error status and message.
/// - `ok`: Creates a new `JsonStatus` with an OK status and optional message.
/// - `data_owned`: Creates a new `JsonStatus` with an OK status and owned data.
/// - `from_anyhow`: Creates a new `JsonStatus` from an `anyhow::Error`.
///
/// # Examples
///
/// ```rust
/// let status = JsonStatus::new_owned(Status::Ok, my_data);
/// let empty_status = JsonStatus::new_empty(Status::Ok);
/// let error_status = JsonStatus::error("An error occurred");
/// let ok_status = JsonStatus::ok(Some("Operation successful"));
/// let data_status = JsonStatus::data_owned(my_data);
/// let anyhow_status = JsonStatus::from_anyhow(anyhow::anyhow!("An error occurred"));
/// ```
/// Implementation of `JsonStatus` for handling JSON responses with different statuses.
impl JsonStatus<'static> {
    /// Creates a new `JsonStatus` with the given status and owned data.
    ///
    /// # Arguments
    ///
    /// * `status` - The status of the JSON response.
    /// * `data` - The data to be serialized and included in the response.
    ///
    /// # Returns
    ///
    /// A new `JsonStatus` instance with the specified status and data.
    pub fn new_owned(status: Status, data: impl EraSerialize + 'static) -> Self {
        JsonStatus {
            status,
            data: Data::Owned(Box::new(data)),
        }
    }

    /// Creates a new `JsonStatus` with the given status and an empty data vector.
    ///
    /// # Arguments
    ///
    /// * `status` - The status of the JSON response.
    ///
    /// # Returns
    ///
    /// A new `JsonStatus` instance with the specified status and an empty data vector.
    pub fn new_empty(status: Status) -> Self {
        JsonStatus {
            status,
            data: Data::Owned(Box::new(Vec::<u8>::new())),
        }
    }

    /// Creates a new `JsonStatus` with an error status and the given error message.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message to be included in the response.
    ///
    /// # Returns
    ///
    /// A new `JsonStatus` instance with an error status and the specified error message.
    pub fn error<T: ToString>(message: T) -> Self {
        JsonStatus {
            status: Status::Error(message.to_string()),
            data: Data::Owned(Box::new(Vec::<u8>::new())),
        }
    }

    /// Creates a new `JsonStatus` with an OK status and an optional message.
    ///
    /// # Arguments
    ///
    /// * `message` - An optional message to be included in the response.
    ///
    /// # Returns
    ///
    /// A new `JsonStatus` instance with an OK status and the specified message, if provided.
    pub fn ok<T: ToString>(message: Option<T>) -> Self {
        JsonStatus {
            status: Status::Ok,
            data: Data::Owned(Box::new(
                message.map_or(Vec::<u8>::new(), |msg| msg.to_string().into_bytes()),
            )),
        }
    }

    /// Creates a new `JsonStatus` with an OK status and the given owned data.
    ///
    /// # Arguments
    ///
    /// * `data` - The data to be serialized and included in the response.
    ///
    /// # Returns
    ///
    /// A new `JsonStatus` instance with an OK status and the specified data.
    pub fn data_owned(data: impl EraSerialize + 'static) -> Self {
        JsonStatus {
            status: Status::Ok,
            data: Data::Owned(Box::new(data)),
        }
    }

    /// Creates a new `JsonStatus` from an `anyhow::Error`.
    ///
    /// # Arguments
    ///
    /// * `error` - The error to be included in the response.
    ///
    /// # Returns
    ///
    /// A new `JsonStatus` instance with an error status and the specified error message.
    pub fn from_anyhow(error: anyhow::Error) -> Self {
        JsonStatus::error(format!("{:?}", error))
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for JsonStatus<'o> {
    /// Converts the `JsonStatus` into a `RawJson` response and delegates the response to it.
    ///
    /// # Arguments
    ///
    /// * `self` - The `JsonStatus` instance to be converted.
    /// * `request` - The incoming Rocket request.
    ///
    /// # Returns
    ///
    /// A `Result` containing the response to be sent back to the client.
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        RawJson::<String>::from(self).respond_to(request)
    }
}

impl<'a> JsonStatus<'a> {
    /// Creates a new `JsonStatus` with the specified status and a reference to serializable data.
    ///
    /// # Arguments
    /// * `status` - The status code to include in the response
    /// * `data` - A reference to any type that's `Serialize`
    ///
    /// # Returns
    /// A new `JsonStatus` instance containing the provided status and data reference
    ///
    /// # Example
    /// ```
    /// let my_data = MySerializableStruct::new();
    /// let response = JsonStatus::new_ref(Status::Created, &my_data);
    /// ```
    pub fn new_ref(status: Status, data: &'a dyn EraSerialize) -> Self {
        JsonStatus {
            status,
            data: Data::Ref(data),
        }
    }

    /// Creates a new `JsonStatus` with an "OK" status and the provided data reference.
    ///
    /// This is a convenience method that automatically sets the status to `Status::Ok`.
    ///
    /// # Type Parameters
    /// * `T` - Any type that's `Serialize`
    ///
    /// # Arguments
    /// * `data` - A reference to the data to be included in the response
    ///
    /// # Returns
    /// A new `JsonStatus` instance with `Status::Ok` and the provided data reference
    ///
    /// # Example
    /// ```
    /// let my_data = MySerializableStruct::new();
    /// let response = JsonStatus::data_ref(&my_data);
    /// ```
    pub fn data_ref<T: EraSerialize>(data: &'a T) -> Self {
        JsonStatus::new_ref(Status::Ok, data)
    }
}
