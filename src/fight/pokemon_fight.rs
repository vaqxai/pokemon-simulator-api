use anyhow::Result;

use super::{FightEvent, FightLog};
use crate::pokemon::Pokemon;

/// Process a fight between two pokemon with a given amount of HP and return a log of the battle
pub async fn process_fight_with_hp(
    contender: &Pokemon,
    challenger: &Pokemon,
    mut contender_hp: u32,
    mut challenger_hp: u32
) -> Result<FightLog> {
    let starting_pokemon = if contender.stats.agility > challenger.stats.agility {
        contender
    } else {
        challenger
    };

    let contender_primary_type = contender.primary_type().clone().resolve().await?;
    let contender_secondary_type = match contender.secondary_type().map(|t| t.clone().resolve()) {
        Some(t) => Some(t.await?),
        None => None,
    };

    let challenger_primary_type = challenger.primary_type().clone().resolve().await?;
    let challenger_secondary_type = match challenger.secondary_type().map(|t| t.clone().resolve()) {
        Some(t) => Some(t.await?),
        None => None,
    };

    let mut last_to_attack = starting_pokemon.clone();

    let mut log = FightLog {
        contender_name: contender.name.clone(),
        challenger_name: challenger.name.clone(),
        log: vec![],
    };

    while contender_hp > 0 && challenger_hp > 0 {
        let (attacker, atk_ptype, atk_stype, atk_hp, defender, def_ptype, def_stype, mut def_hp) =
            if &last_to_attack == challenger {
                (
                    contender,
                    &contender_primary_type,
                    &contender_secondary_type,
                    contender_hp as f32,
                    challenger,
                    &challenger_primary_type,
                    &challenger_secondary_type,
                    challenger_hp as f32,
                )
            } else {
                (
                    challenger,
                    &challenger_primary_type,
                    &challenger_secondary_type,
                    challenger_hp as f32,
                    contender,
                    &contender_primary_type,
                    &contender_secondary_type,
                    contender_hp as f32,
                )
            };

        let mut damage_mult: f32 = 1.0;

        // Calculate primary vs primary type advantage
        if atk_ptype.is_strong_against(def_ptype) {
            damage_mult += 0.375
        } else if atk_ptype.is_weak_against(def_ptype) {
            damage_mult -= 0.225
        }

        // Calculate secondary vs primary type advantage
        if let Some(def_stype) = def_stype {
            if atk_ptype.is_strong_against(def_stype) {
                damage_mult += 0.375
            } else if atk_ptype.is_weak_against(def_stype) {
                damage_mult -= 0.225
            }
        }

        // Calculate primary vs secondary type advantage
        if let Some(atk_stype) = atk_stype {
            if def_ptype.is_strong_against(atk_stype) {
                damage_mult -= 0.225
            } else if def_ptype.is_weak_against(atk_stype) {
                damage_mult += 0.375
            }
        }

        // Calculate secondary vs secondary type advantage
        if let Some(atk_stype) = atk_stype {
            if let Some(def_stype) = def_stype {
                if def_stype.is_strong_against(atk_stype) {
                    damage_mult -= 0.225
                } else if def_stype.is_weak_against(atk_stype) {
                    damage_mult += 0.375
                }
            }
        }

        // total max dmg mult = 2.5
        // total min dmg mult = 0.1

        // 0.8 - 1.2
        let rand_mult = 0.8 + (rand::random::<f32>() * 0.4);

        let defense_mult = 1.0 - ((defender.stats.defense as f32 / 100.0) * 0.5);

        let damage = ((attacker.stats.attack as f32 * damage_mult) * rand_mult) * defense_mult;

        def_hp -= damage;

        let event = FightEvent::Hit {
            attacker: attacker.name.clone(),
            defender: defender.name.clone(),
            damage: damage as u32,
        };

        log.log.push(event);

        if def_hp <= 0.0 {
            let event = FightEvent::Fainted {
                pokemon: defender.name.clone(),
            };

            log.log.push(event);

            let event = FightEvent::PokemonWinner {
                pokemon: attacker.name.clone(),
                hp_left: atk_hp.round() as u32,
            };

            log.log.push(event);
        }

        last_to_attack = attacker.clone();

        if &last_to_attack == challenger {
            contender_hp = def_hp.round() as u32;
            challenger_hp = atk_hp.round() as u32;
        } else {
            contender_hp = atk_hp.round() as u32;
            challenger_hp = def_hp.round() as u32;
        }
    }

    Ok(log)
}

/// Processes a fight between two Pokemon and returns a log of the battle
pub async fn process_fight(contender: &Pokemon, challenger: &Pokemon) -> Result<FightLog> {
    let contender_hp = contender.stats.hp;
    let challenger_hp = challenger.stats.hp;

    process_fight_with_hp(contender, challenger, contender_hp, challenger_hp).await
}
