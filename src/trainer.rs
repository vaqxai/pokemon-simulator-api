use serde::{Deserialize, Serialize};

use crate::{database::promise::Promise, pokemon::Pokemon};

/// Represents a Pokémon trainer with a name and a team of Pokémon
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Trainer {
    /// The name of the trainer
    pub name: String,
    /// The team of Pokemon owned by the trainer
    pub team: Vec<Promise<Pokemon>>,
}
