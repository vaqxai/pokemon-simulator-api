use core::str;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::pokemon::Pokemon;

/// HTTP Enpoints for simulating pokemon and trainer fights
pub mod endpoints;

/// A module for simulating a fight between two pokemon
pub mod pokemon_fight;

/// A module for simulating a fight between trainers
pub mod trainer_fight;

/// Represents a fight event that can occur during a Pokemon battle
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "event_name", content = "event_data")]
pub enum FightEvent {
    /// A Pokemon is chosen by a trainer to fight
    ChoosePokemon {
        /// The trainer who chose the Pokemon
        trainer: String,
        /// The name of the pokemon
        pokemon: String,
    },
    /// A Pokemon attacks another Pokemon
    Hit {
        /// The name of the attacking Pokemon
        attacker: String,
        /// The name of the defending Pokemon
        defender: String,
        /// The amount of damage dealt
        damage: u32,
        /// The amount of HP left on the defending Pokemon
        hp_left: u32,
    },
    /// A Pokemon faints
    Fainted {
        /// The name of the Pokemon that fainted
        pokemon: String,
    },
    /// A Pokemon wins the battle
    PokemonWinner {
        /// The name of the winning Pokemon
        pokemon: String,
        /// The amount of HP left on the winning Pokemon
        hp_left: u32,
    },
    /// A trainer wins the battle
    Winner {
        /// The name of the winning trainer
        trainer: String,
        /// The names of the Pokemon left on the winning trainer's team
        pokemon_left: Vec<String>,
    },
}

/// Represents a log of a Pokemon battle
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FightLog {
    contender_name: String,
    challenger_name: String,
    log: Vec<FightEvent>,
}

/// Represents a trainer's strategy during a fight
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FightStrategy {
    /// Choose the pokemon that has the highest attack stat
    StrongestAtk,

    /// Choose the pokemon that has the highest defense stat
    StrongestDef,

    /// Choose the pokemon that has the highest attack+defense sum
    StrongestSum,

    /// Choose the pokemon that'll have a type advantage
    /// or if not possible, choose the strongest-sum pokemon
    StrongestType,

    /// Choose a random pokemon
    Random,
}

impl FromStr for FightStrategy {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "StrongestAtk" => Ok(FightStrategy::StrongestAtk),
            "StrongestDef" => Ok(FightStrategy::StrongestDef),
            "StrongestSum" => Ok(FightStrategy::StrongestSum),
            "StrongestType" => Ok(FightStrategy::StrongestType),
            "Random" => Ok(FightStrategy::Random),
            _ => Err(()),
        }
    }
}

impl FightStrategy {
    /// Chooses a pokemon from a team based on the strategy
    pub async fn choose_pokemon(
        &self,
        team: &[Pokemon],
        enemy_pokemon: Option<&Pokemon>,
    ) -> Option<Pokemon> {
        match self {
            FightStrategy::StrongestAtk => team.iter().max_by_key(|p| p.stats.attack).cloned(),
            FightStrategy::StrongestDef => team.iter().max_by_key(|p| p.stats.defense).cloned(),
            FightStrategy::StrongestSum => team
                .iter()
                .max_by_key(|p| p.stats.attack + p.stats.defense)
                .cloned(),
            FightStrategy::StrongestType => {
                // 1. Find out if we have a pokemon that has "strong against" both enemy types
                // 2. Find out if we have a pokemon that has "strong against" one enemy type but no weak against
                // 3. Find out if we have a pokemon that has "strong against" one enemy type
                // 4. Find out if we have a pokemon that has no "weak against" enemy types
                // 5. Find out if we have a pokemon that has only one "weak_against type"
                // 6. Choose the strongest sum pokemon

                // If enemy types can't be determined we use the strongest-sum strategy

                let enemy_pokemon = match enemy_pokemon {
                    Some(p) => p,
                    None => return team.iter().max_by_key(|p| p.stats.attack + p.stats.defense).cloned(),
                };

                let enemy_primary_type = match enemy_pokemon.primary_type().clone().resolve().await
                {
                    Ok(t) => Some(t),
                    Err(_) => None,
                };

                let enemy_secondary_type = match enemy_pokemon.secondary_type() {
                    Some(t) => t.clone().resolve().await.ok(),
                    None => None,
                };

                for pokemon in team.iter() {
                    let own_primary_type = match pokemon.primary_type().clone().resolve().await {
                        Ok(t) => t,
                        Err(_) => continue,
                    };

                    let own_secondary_type = match pokemon.secondary_type() {
                        Some(t) => t.clone().resolve().await.ok(),
                        None => None,
                    };

                    if let Some(enemy_primary_type) = &enemy_primary_type {
                        if own_primary_type.is_strong_against(enemy_primary_type) {
                            if let Some(enemy_secondary_type) = &enemy_secondary_type {
                                if own_primary_type.is_strong_against(enemy_secondary_type) {
                                    return Some(pokemon.clone());
                                }
                            } else {
                                return Some(pokemon.clone());
                            }
                        }
                    }

                    if let Some(enemy_secondary_type) = &enemy_secondary_type {
                        if own_primary_type.is_strong_against(enemy_secondary_type) {
                            return Some(pokemon.clone());
                        }
                    }

                    if let Some(own_secondary_type) = &own_secondary_type {
                        if let Some(enemy_primary_type) = &enemy_primary_type {
                            if own_secondary_type.is_strong_against(enemy_primary_type) {
                                return Some(pokemon.clone());
                            }
                        }

                        if let Some(enemy_secondary_type) = &enemy_secondary_type {
                            if own_secondary_type.is_strong_against(enemy_secondary_type) {
                                return Some(pokemon.clone());
                            }
                        }
                    }

                    if let Some(enemy_primary_type) = &enemy_primary_type {
                        if own_primary_type.is_weak_against(enemy_primary_type) {
                            continue;
                        }
                    }

                    if let Some(enemy_secondary_type) = &enemy_secondary_type {
                        if own_primary_type.is_weak_against(enemy_secondary_type) {
                            continue;
                        }
                    }

                    if let Some(own_secondary_type) = own_secondary_type {
                        if let Some(enemy_primary_type) = &enemy_primary_type {
                            if own_secondary_type.is_weak_against(enemy_primary_type) {
                                continue;
                            }
                        }

                        if let Some(enemy_secondary_type) = &enemy_secondary_type {
                            if own_secondary_type.is_weak_against(enemy_secondary_type) {
                                continue;
                            }
                        }
                    }

                    return Some(pokemon.clone());
                }

                team.iter()
                    .max_by_key(|p| p.stats.attack + p.stats.defense)
                    .cloned()
            }
            FightStrategy::Random => {
                let idx = rand::random::<usize>() % team.len();
                team.get(idx).cloned()
            }
        }
    }
}
