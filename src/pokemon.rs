use std::pin::Pin;

use serde::{Deserialize, Serialize};

use crate::database::{AsDbString, DbDelete, DbGet, DbLink, DbPut, DbRepr, DbUpdate};

use anyhow::Result;

/// Represents a Pokemon entity with its basic attributes and stats
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Pokemon {
    /// The name of the Pokemon
    pub name: String,

    /// The primary type of the Pokemon
    /// This field is not public because it should be set by database link
    /// operations only
    /// Use the new fn to construct a Pokemon with types
    primary_type: DbPromise<PokemonType>,

    /// The secondary type of the Pokemon
    /// This field is not public because it should be set by database link
    /// operations only
    /// Use the new fn to construct a Pokemon with types
    secondary_type: Option<DbPromise<PokemonType>>,
    /// The base stats of the Pokemon
    pub stats: PokemonStats,
}

impl DbRepr for Pokemon {
    const DB_NODE_KIND: &'static str = "Pokemon";
    const DB_IDENTIFIER_FIELD: &'static str = "name";

    fn get_identifier(&self) -> String {
        format!("'{}'", self.name)
    }
}

/// Does not include types, which must be linked as relationships
impl DbPut for Pokemon {
    fn put_args(&self) -> String {
        format!(
            "{{ name: '{}', hp: {}, attack: {}, defense: {}, agility: {} }}",
            self.name, self.stats.hp, self.stats.attack, self.stats.defense, self.stats.agility
        )
    }
}

impl DbDelete for Pokemon {}

impl Pokemon {
    /// Creates a new pokemon, places it in the database
    /// Does nothing on duplicate
    /// and links its types to the database
    pub async fn new_to_db(
        name: String,
        primary_type: PokemonType,
        secondary_type: Option<PokemonType>,
        stats: PokemonStats,
    ) -> Result<Self> {
        let mut new = Self {
            name,
            primary_type,
            secondary_type,
            stats,
        };

        // put the pokemon in the db
        new.put_self().await?;

        // link types to db
        new.link_to(
            &new.primary_type.clone(),
            &PokemonPokemonTypeRelationship::PrimaryType,
        )
        .await?;
        if let Some(secondary_type) = &new.secondary_type {
            new.link_to(
                &secondary_type.clone(),
                &PokemonPokemonTypeRelationship::SecondaryType,
            )
            .await?;
        }

        Ok(new)
    }

    /// Change the secondary type of a pokemon
    /// This is possible because the secondary type is an Option
    pub async fn set_secondary_type(
        &mut self,
        new_secondary_type: Option<PokemonType>,
    ) -> Result<()> {
        if self.secondary_type.is_some() {
            self.unlink_from(
                &self.secondary_type.clone().unwrap(),
                &PokemonPokemonTypeRelationship::SecondaryType,
            )
            .await?;
        }

        match new_secondary_type {
            Some(nst) => {
                self.link_to(&nst, &PokemonPokemonTypeRelationship::SecondaryType)
                    .await?;

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
                &PokemonPokemonTypeRelationship::PrimaryType,
                format!("'{}'", identifier.clone()),
            )
            .await?
            .into_iter()
            .next()
            .ok_or(anyhow::anyhow!("No primary type found for Pokemon"))?;
            let secondary_type = Self::get_linked_by_id(
                &PokemonPokemonTypeRelationship::SecondaryType,
                format!("'{}'", identifier.clone()),
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
}

/// Represents the relationship between a Pokemon and its type
pub enum PokemonPokemonTypeRelationship {
    /// Represents the primary type of a Pokemon
    PrimaryType,
    /// Represents the secondary type of a Pokemon
    SecondaryType,
}

impl AsDbString for PokemonPokemonTypeRelationship {
    fn as_db_string(&self) -> &'static str {
        match self {
            PokemonPokemonTypeRelationship::PrimaryType => "PrimaryType",
            PokemonPokemonTypeRelationship::SecondaryType => "SecondaryType",
        }
    }
}

impl DbLink<PokemonType> for Pokemon {
    type RelationshipType = PokemonPokemonTypeRelationship;

    fn link_side_effect(
        &mut self,
        other: &PokemonType,
        relationship_type: &Self::RelationshipType,
    ) -> Result<()> {
        match relationship_type {
            PokemonPokemonTypeRelationship::PrimaryType => {
                self.primary_type = other.clone();
                Ok(())
            }
            PokemonPokemonTypeRelationship::SecondaryType => {
                self.secondary_type = Some(other.clone());
                Ok(())
            }
        }
    }

    fn unlink_side_effect(
        &mut self,
        _other: &PokemonType,
        relationship_type: &Self::RelationshipType,
    ) -> Result<()> {
        match relationship_type {
            PokemonPokemonTypeRelationship::PrimaryType => {
                // TODO: Implement a way to change a pokemon's primary type
                Err(anyhow::anyhow!("Primary type cannot be unlinked"))
            }
            PokemonPokemonTypeRelationship::SecondaryType => {
                self.secondary_type = None;
                Ok(())
            }
        }
    }
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
    /// This field is not public because it should be set by database link
    strong_against: Vec<DbPromise<PokemonType>>,

    /// The types that this Pokemon type is weak against
    /// This field is not public because it should be set by database link
    weak_against: Vec<DbPromise<PokemonType>>,
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

        new.put_self().await?;

        Ok(new)
    }
}

impl DbRepr for PokemonType {
    const DB_NODE_KIND: &'static str = "PokemonType";
    const DB_IDENTIFIER_FIELD: &'static str = "name";

    fn get_identifier(&self) -> String {
        format!("'{}'", self.name)
    }
}

/// Represents the relationship between two Pokemon types
pub enum PokemonTypeRelationship {
    /// Represents a type that is strong against another type
    StrongAgainst,
    /// Represents a type that is weak against another type
    WeakAgainst,
}

impl AsDbString for PokemonTypeRelationship {
    fn as_db_string(&self) -> &'static str {
        match self {
            PokemonTypeRelationship::StrongAgainst => "StrongAgainst",
            PokemonTypeRelationship::WeakAgainst => "WeakAgainst",
        }
    }
}

impl DbLink<PokemonType> for PokemonType {
    type RelationshipType = PokemonTypeRelationship;

    fn link_side_effect(
        &mut self,
        other: &PokemonType,
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
        other: &PokemonType,
        relationship_type: &Self::RelationshipType,
    ) -> Result<()> {
        match relationship_type {
            Self::RelationshipType::StrongAgainst => {
                self.strong_against.retain(|t| t.name != other.name);
                Ok(())
            }
            Self::RelationshipType::WeakAgainst => {
                self.weak_against.retain(|t| t.name != other.name);
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

            let strong_against = new
                .get_linked_to(&PokemonTypeRelationship::StrongAgainst)
                .await?;
            let weak_against = new
                .get_linked_to(&PokemonTypeRelationship::WeakAgainst)
                .await?;

            new.strong_against = strong_against;
            new.weak_against = weak_against;

            Ok(new)
        })
    }
}

impl DbPut for PokemonType {
    fn put_args(&self) -> String {
        format!("{{name: '{}'}}", self.name)
    }
}

impl DbUpdate for PokemonType {
    fn update_args(&self) -> String {
        format!("SET name = '{}'", self.name)
    }
}

impl DbDelete for PokemonType {}
