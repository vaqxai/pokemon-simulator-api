/// Trainer HTTP endpoints module
pub mod endpoints;

use serde::{Deserialize, Serialize};

use anyhow::Result;

use crate::{
    database::{
        AsDbString, DbRepr, delete::DbDelete, get::DbGet, link::DbLink, promise::Promise,
        put::DbPut, sanitize,
    },
    pokemon::Pokemon,
};

/// Represents a Pokémon trainer with a name and a team of Pokémon
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Trainer {
    /// The name of the trainer
    pub name: String,
    /// The team of Pokemon owned by the trainer
    pub team: Vec<Promise<Pokemon>>,
}

impl DbRepr for Trainer {
    const DB_IDENTIFIER_FIELD: &'static str = "name";
    const DB_NODE_KIND: &'static str = "Trainer";

    fn get_identifier(&self) -> String {
        self.name.clone()
    }
}

impl DbPut for Trainer {
    fn put_args(&self) -> String {
        format!("name: '{}'", sanitize(&self.name))
    }
}

/// Represents a relationship between a trainer and a Pokemon
pub enum Relationship {
    /// The trainer owns the Pokemon
    Owns,
}

impl AsDbString for Relationship {
    fn as_db_string(&self) -> &'static str {
        match self {
            Relationship::Owns => "Owns",
        }
    }
}

impl DbLink<Pokemon> for Trainer {
    type RelationshipType = Relationship;

    fn link_side_effect(
        &mut self,
        pokemon: &Promise<Pokemon>,
        relationship: &Self::RelationshipType,
    ) -> Result<()> {
        match relationship {
            Relationship::Owns => {
                self.team.push(pokemon.clone());
                Ok(())
            }
        }
    }

    fn unlink_side_effect(
        &mut self,
        pokemon: &Promise<Pokemon>,
        relationship: &Self::RelationshipType,
    ) -> Result<()> {
        match relationship {
            Relationship::Owns => {
                self.team.retain(|p| p.ident() != pokemon.ident());
                Ok(())
            }
        }
    }
}

impl DbGet for Trainer {
    fn from_db_node(node: neo4rs::Node) -> Self::Future
    where
        Self: Sized,
    {
        Box::pin(async move {
            let name = node.get::<String>("name")?;

            let team =
                Trainer::get_linked_by_id(&Relationship::Owns, format!("'{}'", sanitize(&name)))
                    .await?;

            Ok(Trainer { name, team })
        })
    }

    fn identifier_from_node(node: neo4rs::Node) -> String
    where
        Self: Sized,
    {
        node.get::<String>("name").unwrap()
    }
}

impl DbDelete for Trainer {}
