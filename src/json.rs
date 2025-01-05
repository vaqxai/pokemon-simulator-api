use erased_serde::Serialize as EraSerialize;
use rocket::response::Responder;
use rocket::response::content::RawJson;
use serde::Serialize;

#[derive(Serialize)]
pub enum Status {
    Ok,
    Error(String),
}

#[derive(Serialize)]
#[serde(untagged)]
enum Data<'a> {
    Owned(Box<dyn EraSerialize + 'static>),
    Ref(&'a dyn EraSerialize),
}

#[derive(Serialize)]
pub struct JsonStatus<'a> {
    status: Status,
    data: Data<'a>,
}

impl From<JsonStatus<'_>> for RawJson<String> {
    fn from(status: JsonStatus) -> Self {
        RawJson(serde_json::to_string(&status).unwrap())
    }
}

impl<T: std::error::Error> From<T> for JsonStatus<'static> {
    fn from(error: T) -> Self {
        info!("Error while running request: {}", error);
        JsonStatus::error(format!("{}", error))
    }
}

pub type JsonResult<'a> = Result<JsonStatus<'a>, JsonStatus<'static>>;

impl JsonStatus<'static> {
    pub fn new_owned(status: Status, data: impl EraSerialize + 'static) -> Self {
        JsonStatus {
            status,
            data: Data::Owned(Box::new(data)),
        }
    }

    pub fn new_empty(status: Status) -> Self {
        JsonStatus::new_owned(status, vec![""])
    }

    pub fn error<T: ToString>(message: T) -> Self {
        info!("Error while running request: {}", message.to_string());
        JsonStatus::new_empty(Status::Error(message.to_string()))
    }

    pub fn ok<T: ToString>(message: Option<T>) -> Self {
        match message {
            Some(message) => JsonStatus::new_owned(Status::Ok, vec![message.to_string()]),
            None => JsonStatus::new_empty(Status::Ok),
        }
    }

    pub fn data_owned(data: impl EraSerialize + 'static) -> Self {
        JsonStatus::new_owned(Status::Ok, data)
    }

    pub fn from_anyhow(error: anyhow::Error) -> Self {
        info!("Error while running request: {}", error);
        JsonStatus::error(format!("{}", error))
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for JsonStatus<'o> {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'o> {
        RawJson::<String>::from(self).respond_to(request)
    }
}

impl<'a> JsonStatus<'a> {
    pub fn new_ref(status: Status, data: &'a dyn EraSerialize) -> Self {
        JsonStatus {
            status,
            data: Data::Ref(data),
        }
    }

    pub fn data_ref<T: EraSerialize>(data: &'a T) -> Self {
        JsonStatus::new_ref(Status::Ok, data)
    }
}
