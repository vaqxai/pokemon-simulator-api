use crate::{
    database::get::DbGet,
    fight,
    json::{JsonResult, JsonStatus},
    pokemon::Pokemon,
};

/// Endpoint to simulate a fight between two Pokemon.
#[get("/simulate_fight/<contender_name>/<challenger_name>")]
pub async fn simulate_fight<'a>(contender_name: String, challenger_name: String) -> JsonResult<'a> {
    info!(
        "Request to /api/simulate_fight/{}/{}",
        contender_name, challenger_name
    );

    let contender = match Pokemon::get_first(&contender_name).await {
        Ok(pokemon) => pokemon,
        Err(_) => return Err(JsonStatus::error("Contender not found")),
    };

    let challenger = match Pokemon::get_first(&challenger_name).await {
        Ok(pokemon) => pokemon,
        Err(_) => return Err(JsonStatus::error("Challenger not found")),
    };

    let log = fight::process_fight(&contender, &challenger)
        .await
        .map_err(JsonStatus::from_anyhow)?;

    Ok(JsonStatus::data_owned(log))
}
