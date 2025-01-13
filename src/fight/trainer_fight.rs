use crate::trainer::Trainer;
use anyhow::{anyhow, Result};
use crate::pokemon::Pokemon;
use super::{FightEvent, FightLog, FightStrategy};

async fn process_victory(winner_name: String, winner_team: &[Pokemon]) -> FightEvent {
    FightEvent::Winner {
        trainer: winner_name,
        pokemon_left: winner_team.iter().map(|p| p.name.clone()).collect(),
    }
}

/// Process a fight between two trainers and return a log of the battle
pub async fn process_fight(
    challenger: &Trainer,
    contender: &Trainer,
    challenger_strat: FightStrategy,
    contender_strat: FightStrategy,
) -> Result<FightLog> {
    // Resolve all pokemon of each team
    let mut challenger_team =
        futures::future::try_join_all(challenger.team.iter().map(|p| p.clone().resolve())).await?;

    let mut contender_team =
        futures::future::try_join_all(contender.team.iter().map(|p| p.clone().resolve())).await?;

    // Create a log of the fight
    let mut log = FightLog {
        contender_name: contender.name.clone(),
        challenger_name: challenger.name.clone(),
        log: vec![],
    };

    // Fight until one of the teams has no more pokemon
    // the contender is the first to choose their pokemon using their strategy.

    // contender chooses their pokemon
    let mut contender_pokemon = match
    contender_strat.choose_pokemon(&contender_team, None).await {
        Some(p) => Some(p),
        None => return Err(anyhow::anyhow!("Contender's strategy produced no valid pokemon")),
    };

    // add this to the log
    log.log.push(FightEvent::ChoosePokemon {
        trainer: contender.name.clone(),
        pokemon: contender_pokemon.as_ref().unwrap().name.clone(),
    });

    // challenger chooses their pokemon
    let mut challenger_pokemon = match
    challenger_strat.choose_pokemon(&challenger_team, Some(&contender_pokemon.as_ref().unwrap())).await {
        Some(p) => Some(p),
        None => return Err(anyhow::anyhow!("Challenger's strategy produced no valid pokemon")),
    };

    // add this to the log
    log.log.push(FightEvent::ChoosePokemon {
        trainer: challenger.name.clone(),
        pokemon: challenger_pokemon.as_ref().unwrap().name.clone(),
    });

    // Set the HP of the pokemon
    let mut contender_hp = contender_pokemon.as_ref().unwrap().stats.hp;
    let mut challenger_hp = challenger_pokemon.as_ref().unwrap().stats.hp;

    while !challenger_team.is_empty() && !contender_team.is_empty() {
        
        let mut should_remove_challenger = false;
        let mut should_remove_contender = false;

         match (&mut challenger_pokemon, &mut contender_pokemon) {
            (Some(chal_poke), None) => {
                // choose a new pokemon for the contender or end the game
                if challenger_team.is_empty() {
                    // challenger has no more pokemon, contender wins
                    log.log.push(process_victory(contender.name.clone(), &contender_team).await);
                }

                contender_pokemon = match
                contender_strat.choose_pokemon(&contender_team, Some(&chal_poke)).await {
                    Some(p) => Some(p),
                    None => return Err(anyhow::anyhow!("Contender's strategy produced no valid pokemon")),
                };
                log.log.push(FightEvent::ChoosePokemon {
                    trainer: contender.name.clone(),
                    pokemon: contender_pokemon.as_ref().unwrap().name.clone(),
                });
            },
            (None, Some(cont_poke)) => {
                // choose a new pokemon for the challenger or end the game
                if contender_team.is_empty() {
                    // contender has no more pokemon, challenger wins
                    log.log.push(process_victory(challenger.name.clone(), &challenger_team).await);
                }

                challenger_pokemon = match
                challenger_strat.choose_pokemon(&challenger_team, Some(&cont_poke)).await {
                    Some(p) => Some(p),
                    None => return Err(anyhow::anyhow!("Challenger's strategy produced no valid pokemon")),
                };
                log.log.push(FightEvent::ChoosePokemon {
                    trainer: challenger.name.clone(),
                    pokemon: challenger_pokemon.as_ref().unwrap().name.clone(),
                });
            },
            (None, None) => {
                // Both cannot be fainted, this situation should not occur
                return Err(anyhow::anyhow!("Both pokemon are fainted, which should not be possible"));
            }
            (Some(chal_poke), Some(cont_poke)) => {
                // fight
                let mut fight_log = super::pokemon_fight::process_fight_with_hp(
                    chal_poke,
                    cont_poke,
                    challenger_hp,
                    contender_hp
                ).await?;

                // get one-before-last item to find out who fainted
                let pre_last_fight_event = fight_log.log.get(fight_log.log.len() - 2).ok_or(anyhow!("No fight events returned from pokemon fight"))?;

                match pre_last_fight_event {
                    FightEvent::Fainted { pokemon} => {
                        // remove the fainted pokemon from the team
                        if chal_poke.name == *pokemon {
                            challenger_team.retain(|p| p.name != *pokemon);
                            should_remove_challenger = true;
                        } else if cont_poke.name == *pokemon {
                            contender_team.retain(|p| p.name != *pokemon);
                            should_remove_contender = true;
                        }
                    }
                    _ => {
                        return Err(anyhow::anyhow!("The pre-last fight event was not a fainted event"));
                    }
                }

                // get the last fight event to find out who won
                let last_fight_event = fight_log.log.last().ok_or(anyhow!("No fight events returned from pokemon fight"))?;

                match last_fight_event {
                    FightEvent::PokemonWinner {
                        pokemon,
                        hp_left,
                    } => {
                        // and set the hp of the remaining pokemon accordingly, to carry it over to the next fight
                        if chal_poke.name == *pokemon {
                            challenger_hp = *hp_left;
                        } else if cont_poke.name == *pokemon {
                            contender_hp = *hp_left;
                        }
                    }
                    _ => {
                        return Err(anyhow::anyhow!("The last fight event was not a pokemon winner event"));
                    }
                }

                // append the fight log to the main log
                log.log.append(&mut fight_log.log);
            }

        }
        
        if should_remove_challenger {
            challenger_pokemon = None;
        } else if should_remove_contender {
            contender_pokemon = None;
        }
        
    }

    Ok(log)
}
