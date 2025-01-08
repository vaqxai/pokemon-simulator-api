/// Pokemon Type module
pub mod ptype;

/// Pokemon Stats (hp, etc) module
pub mod stats;

use std::pin::Pin;

use serde::{Deserialize, Serialize};
use stats::PokemonStats;

use crate::database::{
    AsDbString, DbRepr,
    delete::DbDelete,
    get::DbGet,
    link::DbLink,
    promise::{Promise, Promised},
    put::DbPut,
    sanitize,
};

use anyhow::Result;

use ptype::PokemonType;

/// Represents a Pokemon entity with its basic attributes and stats
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Pokemon {
    /// The name of the Pokemon
    pub name: String,

    /// The primary type of the Pokemon
    /// This field is not public because it should be set by database link
    /// operations only
    /// Use the new fn to construct a Pokemon with types
    primary_type: Promise<PokemonType>,

    /// The secondary type of the Pokemon
    /// This field is not public because it should be set by database link
    /// operations only
    /// Use the new fn to construct a Pokemon with types
    secondary_type: Option<Promise<PokemonType>>,
    /// The base stats of the Pokemon
    pub stats: PokemonStats,
}

impl DbRepr for Pokemon {
    const DB_NODE_KIND: &'static str = "Pokemon";
    const DB_IDENTIFIER_FIELD: &'static str = "name";

    fn get_identifier(&self) -> String {
        format!("'{}'", sanitize(&self.name))
    }
}

/// Does not include types, which must be linked as relationships
impl DbPut for Pokemon {
    fn put_args(&self) -> String {
        format!(
            "{{ name: '{}', hp: {}, attack: {}, defense: {}, agility: {} }}",
            sanitize(&self.name),
            self.stats.hp,
            self.stats.attack,
            self.stats.defense,
            self.stats.agility
        )
    }
}

impl DbDelete for Pokemon {}

impl Pokemon {
    /// Puts the Pokemon in the database with its types
    pub async fn put_with_relationships(&mut self) -> Result<()> {
        self.put_self_only().await?;

        let prim_type = &self.primary_type.clone();
        let sec_type = self.secondary_type.clone();

        self.link_to(prim_type, &Relationship::PrimaryType).await?;
        if let Some(secondary_type) = &sec_type {
            self.link_to(secondary_type, &Relationship::SecondaryType)
                .await?;
        }
        Ok(())
    }

    /// Returns the primary type of the Pokemon
    pub fn primary_type(&self) -> &Promise<PokemonType> {
        &self.primary_type
    }

    /// Returns the secondary type of the Pokemon if it has one
    pub fn secondary_type(&self) -> Option<&Promise<PokemonType>> {
        self.secondary_type.as_ref()
    }

    /// Creates a new pokemon, places it in the database
    /// Does nothing on duplicate
    /// and links its types to the database
    pub async fn new_to_db(
        name: String,
        primary_type: Promise<PokemonType>,
        secondary_type: Option<Promise<PokemonType>>,
        stats: PokemonStats,
    ) -> Result<Self> {
        let mut new = Self {
            name,
            primary_type,
            secondary_type,
            stats,
        };

        // put the pokemon in the db
        new.put_self_only().await?;

        // link types to db
        new.link_to(&new.primary_type.clone(), &Relationship::PrimaryType)
            .await?;
        if let Some(secondary_type) = &new.secondary_type {
            new.link_to(&secondary_type.clone(), &Relationship::SecondaryType)
                .await?;
        }

        Ok(new)
    }

    /// Change the secondary type of a pokemon
    /// This is possible because the secondary type is an Option
    pub async fn set_secondary_type(
        &mut self,
        new_secondary_type: Option<Promise<PokemonType>>,
    ) -> Result<()> {
        if self.secondary_type.is_some() {
            self.unlink_from(
                &self.secondary_type.clone().unwrap(),
                &Relationship::SecondaryType,
            )
            .await?;
        }

        match new_secondary_type {
            Some(nst) => {
                self.link_to(&nst, &Relationship::SecondaryType).await?;

                self.secondary_type = Some(nst);
            }
            None => {
                self.secondary_type = None;
            }
        }

        Ok(())
    }
}

impl DbGet for Pokemon {
    type Future = Pin<Box<dyn Future<Output = Result<Self>> + Send>>;

    fn from_db_node(node: neo4rs::Node) -> Self::Future {
        Box::pin(async move {
            let identifier = node.get::<String>("name")?;

            let primary_type = Self::get_linked_by_id(
                &Relationship::PrimaryType,
                format!("'{}'", sanitize(&identifier)),
            )
            .await?
            .into_iter()
            .next()
            .ok_or(anyhow::anyhow!("No primary type found for Pokemon"))?;
            let secondary_type = Self::get_linked_by_id(
                &Relationship::SecondaryType,
                format!("'{}'", sanitize(&identifier)),
            )
            .await?
            .into_iter()
            .next();

            Ok(Self {
                name: identifier,
                primary_type,
                secondary_type,
                stats: PokemonStats {
                    hp: node.get("hp")?,
                    attack: node.get("attack")?,
                    defense: node.get("defense")?,
                    agility: node.get("agility")?,
                },
            })
        })
    }

    fn identifier_from_node(node: neo4rs::Node) -> String {
        node.get::<String>("name").unwrap()
    }
}

/// Represents the relationship between a Pokemon and its type
pub enum Relationship {
    /// Represents the primary type of a Pokemon
    PrimaryType,
    /// Represents the secondary type of a Pokemon
    SecondaryType,
}

impl AsDbString for Relationship {
    fn as_db_string(&self) -> &'static str {
        match self {
            Relationship::PrimaryType => "PrimaryType",
            Relationship::SecondaryType => "SecondaryType",
        }
    }
}

impl DbLink<PokemonType> for Pokemon {
    type RelationshipType = Relationship;

    fn link_side_effect(
        &mut self,
        other: &Promise<PokemonType>,
        relationship_type: &Self::RelationshipType,
    ) -> Result<()> {
        match relationship_type {
            Relationship::PrimaryType => {
                self.primary_type = other.clone();
                Ok(())
            }
            Relationship::SecondaryType => {
                self.secondary_type = Some(other.clone());
                Ok(())
            }
        }
    }

    fn unlink_side_effect(
        &mut self,
        _other: &Promise<PokemonType>,
        relationship_type: &Self::RelationshipType,
    ) -> Result<()> {
        match *relationship_type {
            Relationship::PrimaryType => {
                // TODO: Implement a way to change a pokemon's primary type
                Err(anyhow::anyhow!("Primary type cannot be unlinked"))
            }
            Relationship::SecondaryType => {
                self.secondary_type = None;
                Ok(())
            }
        }
    }
}

impl Promised for Pokemon {}
