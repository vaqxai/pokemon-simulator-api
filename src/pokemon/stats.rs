use serde::{Deserialize, Serialize};

/// Represents the base stats of a Pokemon, including HP, attack, defense, and agility
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PokemonStats {
    /// The hit points of the Pokemon
    pub hp: u32,

    /// The attack power of the Pokemon
    pub attack: u32,

    /// The defense power of the Pokemon
    pub defense: u32,

    /// This stat determines attack priority in battle
    pub agility: u32,
}
