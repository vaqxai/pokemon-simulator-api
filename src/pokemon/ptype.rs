use std::pin::Pin;

use serde::{Deserialize, Serialize};

use crate::database::{
    AsDbString, DbRepr,
    delete::DbDelete,
    get::DbGet,
    link::DbLink,
    promise::{MaybePromise, Promised},
    put::DbPut,
    sanitize,
    update::DbUpdate,
};

use anyhow::Result;

/// Represents a type of Pokemon, including its strengths and weaknesses
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PokemonType {
    /// The name of the Pokemon type
    pub name: String,

    /// The types that this Pokemon type is strong against
    /// This field is not public because it should be set by database link
    strong_against: Vec<MaybePromise<PokemonType>>,

    /// The types that this Pokemon type is weak against
    /// This field is not public because it should be set by database link
    weak_against: Vec<MaybePromise<PokemonType>>,
}

impl PokemonType {
    /// Creates a new PokemonType and places it in the database
    /// Does nothing on duplicate
    pub async fn new_to_db(name: String) -> Result<Self> {
        let new = Self {
            name,
            strong_against: vec![],
            weak_against: vec![],
        };

        new.put_self_only().await?;

        Ok(new)
    }

    /// Checks if this Pokemon type is strong against another type
    pub fn is_strong_against(&self, other: &PokemonType) -> bool {
        self.strong_against.iter().any(|t| t.ident() == other.name)
    }

    /// Checks if this Pokemon type is weak against another type
    pub fn is_weak_against(&self, other: &PokemonType) -> bool {
        self.weak_against.iter().any(|t| t.ident() == other.name)
    }
}

impl DbRepr for PokemonType {
    const DB_NODE_KIND: &'static str = "PokemonType";
    const DB_IDENTIFIER_FIELD: &'static str = "name";

    fn get_db_identifier(&self) -> String {
        format!("'{}'", sanitize(&self.name))
    }

    fn get_raw_identifier(&self) -> &str {
        &self.name
    }
}

/// Represents the relationship between two Pokemon types
pub enum Relationship {
    /// Represents a type that is strong against another type
    StrongAgainst,
    /// Represents a type that is weak against another type
    WeakAgainst,
}

impl AsDbString for Relationship {
    fn as_db_string(&self) -> &'static str {
        match self {
            Relationship::StrongAgainst => "StrongAgainst",
            Relationship::WeakAgainst => "WeakAgainst",
        }
    }
}

impl DbLink<PokemonType> for PokemonType {
    type RelationshipType = Relationship;

    fn link_side_effect(
        &mut self,
        other: &MaybePromise<PokemonType>,
        relationship_type: &Self::RelationshipType,
    ) -> Result<()> {
        match relationship_type {
            Self::RelationshipType::StrongAgainst => {
                self.strong_against.push(other.clone());
                Ok(())
            }
            Self::RelationshipType::WeakAgainst => {
                self.weak_against.push(other.clone());
                Ok(())
            }
        }
    }

    fn unlink_side_effect(
        &mut self,
        other: &MaybePromise<PokemonType>,
        relationship_type: &Self::RelationshipType,
    ) -> Result<()> {
        match relationship_type {
            Self::RelationshipType::StrongAgainst => {
                self.strong_against.retain(|t| t.ident() != other.ident());
                Ok(())
            }
            Self::RelationshipType::WeakAgainst => {
                self.weak_against.retain(|t| t.ident() != other.ident());
                Ok(())
            }
        }
    }
}

impl DbGet for PokemonType {
    type Future = Pin<Box<dyn Future<Output = Result<Self>> + Send>>;

    fn from_db_node(node: neo4rs::Node) -> Self::Future {
        Box::pin(async move {
            let mut new = Self {
                name: node.get("name")?,
                strong_against: vec![],
                weak_against: vec![],
            };

            let strong_against = new.get_linked_to(&Relationship::StrongAgainst).await?;
            let weak_against = new.get_linked_to(&Relationship::WeakAgainst).await?;

            new.strong_against = strong_against;
            new.weak_against = weak_against;

            Ok(new)
        })
    }

    /// Panics: if supplied node does not have a "name" field
    fn identifier_from_node(node: neo4rs::Node) -> String
    where
        Self: Sized,
    {
        node.get::<String>("name").unwrap().to_string()
    }
}

impl DbPut for PokemonType {
    fn put_args(&self) -> String {
        format!("{{name: '{}'}}", sanitize(&self.name))
    }
}

impl DbUpdate for PokemonType {
    fn update_args(&self) -> String {
        format!("SET name = '{}'", sanitize(&self.name))
    }
}

impl DbDelete for PokemonType {}

impl Promised for PokemonType {}
