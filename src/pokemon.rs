use serde::{Deserialize, Serialize};

/// Represents a Pokemon entity with its basic attributes and stats
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Pokemon {
    /// The name of the Pokemon
    pub name: String,
    /// The primary type of the Pokemon
    pub primary_type: PokemonType,
    /// The secondary type of the Pokemon, if it has one
    pub secondary_type: Option<PokemonType>,
    /// The base stats of the Pokemon
    pub stats: PokemonStats,
}

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

/// Represents a type of Pokemon, including its strengths and weaknesses
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PokemonType {
    /// The name of the Pokemon type
    pub name: String,

    /// The types that this Pokemon type is strong against
    pub strong_against: Vec<PokemonType>,

    /// The types that this Pokemon type is weak against
    pub weak_against: Vec<PokemonType>,
}
