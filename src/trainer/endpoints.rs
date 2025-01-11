use crate::{
    database::{
        delete::DbDelete,
        get::DbGet,
        link::DbLink,
        promise::{MaybePromise, Promised},
        put::DbPut,
    },
    json::{self, JsonResult, JsonStatus},
    pokemon::Pokemon,
    trainer::{self, Trainer},
};

/// Endpoint for getting a list of all trainers
#[get("/trainers")]
pub async fn get_trainers<'a>() -> JsonResult<'a> {
    info!("Request to /api/trainers");
    let trainers = Trainer::get_all().await.map_err(JsonStatus::from_anyhow)?;
    Ok(JsonStatus::data_owned(trainers))
}

/// Endpoint for getting a list of all Pokemon owned by a trainer.
#[get("/trainer_pokemons/<trainer_name>")]
pub async fn get_trainer_pokemons<'a>(trainer_name: String) -> JsonResult<'a> {
    info!("Request to /api/trainer_pokemons/{}", trainer_name);

    let mut trainer = match Trainer::get_first(&trainer_name).await {
        Ok(trainer) => trainer,
        Err(_) => return Err(JsonStatus::error("Trainer not found")),
    };

    // resolve the trainer's pokemons
    for p in &mut trainer.team {
        *p = MaybePromise::from_concrete(
            p.clone().resolve().await.map_err(JsonStatus::from_anyhow)?,
        );
    }

    Ok(JsonStatus::data_owned(trainer.team))
}

/// Endpoint for creating a new trainer.
#[post("/trainer_pokemons/<trainer_name>")]
pub async fn create_trainer<'a>(trainer_name: String) -> JsonResult<'a> {
    info!("Request to /api/trainer_pokemons/{}", trainer_name);

    if trainer_name.len() > 30 {
        return Err(JsonStatus::error("Name is too long"));
    }

    if trainer_name.is_empty() {
        return Err(JsonStatus::error("Name cannot be empty"));
    }

    let trainer = Trainer {
        name: trainer_name,
        team: vec![],
    };

    trainer
        .put_self_only() // no need for relationships since the team is empty
        .await
        .map_err(JsonStatus::from_anyhow)?;

    Ok(JsonStatus::new_empty(json::Status::Ok))
}

/// Endpoint for deleting a trainer.
#[delete("/trainer_pokemons/<trainer_name>")]
pub async fn delete_trainer<'a>(trainer_name: String) -> JsonResult<'a> {
    info!("Request to /api/trainer_pokemons/{}", trainer_name);

    let mut trainer = match Trainer::get_first(&trainer_name).await {
        Ok(trainer) => trainer,
        Err(_) => return Err(JsonStatus::error("Trainer not found")),
    };

    // remove all links first
    for p in trainer.team.clone() {
        trainer
            .unlink_from(&p, &trainer::Relationship::Owns)
            .await
            .map_err(JsonStatus::from_anyhow)?;
    }

    Trainer::delete(&trainer.name)
        .await
        .map_err(JsonStatus::from_anyhow)?;

    Ok(JsonStatus::new_empty(json::Status::Ok))
}

/// Endpoint for adding a Pokemon to a trainer's team.
#[post("/trainer_pokemons/<trainer_name>/<pokemon_name>")]
pub async fn add_pokemon_to_trainer<'a>(
    trainer_name: String,
    pokemon_name: String,
) -> JsonResult<'a> {
    info!(
        "Request to /api/trainer_pokemons/{}/{}",
        trainer_name, pokemon_name
    );

    let mut trainer = match Trainer::get_first(&trainer_name).await {
        Ok(trainer) => trainer,
        Err(_) => return Err(JsonStatus::error("Trainer not found")),
    };

    let pokemon = match Pokemon::get_first(&pokemon_name).await {
        Ok(pokemon) => pokemon,
        Err(_) => return Err(JsonStatus::error("Pokemon not found")),
    };

    for p in &trainer.team {
        if p.ident() == pokemon.name {
            return Err(JsonStatus::error("Pokemon already in team"));
        }
    }

    trainer
        .link_to(
            &MaybePromise::from_promise(pokemon.as_promise()),
            &trainer::Relationship::Owns,
        )
        .await
        .map_err(JsonStatus::from_anyhow)?;

    Ok(JsonStatus::new_empty(json::Status::Ok))
}

/// Endpoint for removing a Pokemon from a trainer's team.
#[delete("/trainer_pokemons/<trainer_name>/<pokemon_name>")]
pub async fn remove_pokemon_from_trainer<'a>(
    trainer_name: String,
    pokemon_name: String,
) -> JsonResult<'a> {
    info!(
        "Request to /api/trainer_pokemons/{}/{}",
        trainer_name, pokemon_name
    );

    let mut trainer = match Trainer::get_first(&trainer_name).await {
        Ok(trainer) => trainer,
        Err(_) => return Err(JsonStatus::error("Trainer not found")),
    };

    let mut unlink_pokemon = None;
    for p in &trainer.team {
        if p.ident() == pokemon_name {
            unlink_pokemon = Some(p.clone());
            break;
        }
    }

    match unlink_pokemon {
        Some(p) => {
            let rel = trainer::Relationship::Owns;
            trainer
                .unlink_from(&p, &rel)
                .await
                .map_err(JsonStatus::from_anyhow)?;
            Ok(JsonStatus::new_empty(json::Status::Ok))
        }
        None => Err(JsonStatus::error("Pokemon not found in team")),
    }
}
