use crate::{
    database::get::DbGet,
    fight::{pokemon_fight, trainer_fight},
    json::{JsonResult, JsonStatus},
    pokemon::Pokemon,
    trainer::Trainer,
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

    let log = pokemon_fight::process_fight(&contender, &challenger)
        .await
        .map_err(JsonStatus::from_anyhow)?;

    Ok(JsonStatus::data_owned(log))
}

/// Endpoint to simulate a fight between two trainers.
#[get(
    "/simulate_trainer_fight/<challenger_name>/<challenger_strategy>/<contender_name>/<contender_strategy>"
)]
pub async fn simulate_trainer_fight<'a>(
    challenger_name: String,
    challenger_strategy: String,
    contender_name: String,
    contender_strategy: String,
) -> JsonResult<'a> {
    info!(
        "Request to /api/simulate_trainer_fight/{}/{}",
        challenger_name, contender_name
    );

    let challenger = match Trainer::get_first(&challenger_name).await {
        Ok(trainer) => trainer,
        Err(_) => return Err(JsonStatus::error("Challenger not found")),
    };

    let contender = match Trainer::get_first(&contender_name).await {
        Ok(trainer) => trainer,
        Err(_) => return Err(JsonStatus::error("Contender not found")),
    };

    let challenger_strategy = challenger_strategy
        .parse()
        .map_err(|_| JsonStatus::error("Invalid strategy"))?;

    let contender_strategy = contender_strategy
        .parse()
        .map_err(|_| JsonStatus::error("Invalid strategy"))?;

    let log = trainer_fight::process_fight(
        &challenger,
        &contender,
        challenger_strategy,
        contender_strategy,
    )
    .await
    .map_err(JsonStatus::from_anyhow)?;

    Ok(JsonStatus::data_owned(log))
}
