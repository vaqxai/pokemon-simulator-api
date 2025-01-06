use serde::{Deserialize, Serialize};

use crate::database::{DbDelete, DbGet, DbHandle, DbLink, DbPut, DbRepr, DbUpdate};

use anyhow::Result;

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

impl DbRepr for Pokemon {
    const DB_NODE_KIND: &'static str = "Pokemon";
    const DB_IDENTIFIER_FIELD: &'static str = "name";

    fn get_identifier(&self) -> String {
        self.name.clone()
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
    fn get_primary_type_link_query(&self) -> String {
        format!(
            "MATCH (p:Pokemon), (t:PokemonType) WHERE p.name = '{}' AND t.name = '{}' MERGE (p)-[:PrimaryType]->(t);",
            self.name, self.primary_type.name
        )
    }

    fn get_secondary_type_link_query(&self) -> String {
        format!(
            "MATCH (p:Pokemon), (t:PokemonType) WHERE p.name = '{}' AND t.name = '{}' MERGE (p)-[:SecondaryType]->(t);",
            self.name,
            self.secondary_type.as_ref().unwrap().name
        )
    }

    fn remove_primary_type_link_query(&self) -> String {
        format!(
            "MATCH (p:Pokemon)-[r:PrimaryType]->(t:PokemonType) WHERE p.name = '{}' DELETE r;",
            self.name
        )
    }

    fn remove_secondary_type_link_query(&self) -> String {
        format!(
            "MATCH (p:Pokemon)-[r:SecondaryType]->(t:PokemonType) WHERE p.name = '{}' DELETE r;",
            self.name
        )
    }

    /// Creates database relationships for pokemon types
    pub async fn link_types_to_db(&self) -> Result<()> {
        // ensure both types are in the database
        // first find out if primary type is in the db
        let primary_type = PokemonType::get_first(&self.primary_type.name).await;

        if primary_type.is_err() {
            self.primary_type.put_self().await?;
        }

        // then check if secondary type is in the db, but only if we have one
        if let Some(secondary_type) = &self.secondary_type {
            let secondary_type = PokemonType::get_first(&secondary_type.name).await;
            if secondary_type.is_err() {
                self.secondary_type.as_ref().unwrap().put_self().await?;
            }
        }

        // link the types to the pokemon
        let db_handle = DbHandle::connect().await?;

        // check if types aren't linked yet and remove old links if they are
        let mut q_res = db_handle
            .inner
            .execute(self.remove_primary_type_link_query().into())
            .await?;

        let _none = q_res.next().await?;

        let mut q_res = db_handle
            .inner
            .execute(self.remove_secondary_type_link_query().into())
            .await?;

        let _none = q_res.next().await?;

        // link the types to the pokemon
        let mut q_res = db_handle
            .inner
            .execute(self.get_primary_type_link_query().into())
            .await?;

        let _none = q_res.next().await?;

        if let Some(_secondary_type) = &self.secondary_type {
            let mut q_res = db_handle
                .inner
                .execute(self.get_secondary_type_link_query().into())
                .await?;

            let _none = q_res.next().await?;
        }

        Ok(())
    }
}

impl DbGet for Pokemon {
    async fn from_db_node(node: neo4rs::Node) -> Result<Self> {
        Ok(Self {
            name: node.get("name")?,
            primary_type: PokemonType::from_db_node(node.get("primary_type")?)?,
            secondary_type: PokemonType::from_db_node(node.get("primary_type")?)?,
            stats: PokemonStats {
                hp: node.get("hp")?,
                attack: node.get("attack")?,
                defense: node.get("defense")?,
                agility: node.get("agility")?,
            },
        })
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
    pub strong_against: Vec<PokemonType>,

    /// The types that this Pokemon type is weak against
    pub weak_against: Vec<PokemonType>,
}

impl DbRepr for PokemonType {
    const DB_NODE_KIND: &'static str = "PokemonType";
    const DB_IDENTIFIER_FIELD: &'static str = "name";

    fn get_identifier(&self) -> String {
        self.name.clone()
    }
}

impl DbLink<PokemonType> for PokemonType {}

impl DbGet for PokemonType {
    async fn from_db_node(node: neo4rs::Node) -> Result<Self> {
        let mut new = Self {
            name: node.get("name")?,
            strong_against: vec![],
            weak_against: vec![],
        };

        let strong_against = new.get_linked_to("strong_against").await?;
        let weak_against = new.get_linked_to("weak_against").await?;

        new.strong_against = strong_against;
        new.weak_against = weak_against;

        Ok(new)
    }
}

impl DbPut for PokemonType {
    fn put_args(&self) -> String {
        format!(
            "{{name: '{}', strong_against: {}, weak_against: {}}}",
            self.name,
            self.strong_against
                .iter()
                .map(|t| format!("'{}'", t.name))
                .collect::<Vec<String>>()
                .join(", "),
            self.weak_against
                .iter()
                .map(|t| format!("'{}'", t.name))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl DbUpdate for PokemonType {
    fn update_args(&self) -> String {
        format!(
            "SET strong_against = {}, weak_against = {}",
            self.strong_against
                .iter()
                .map(|t| format!("'{}'", t.name))
                .collect::<Vec<String>>()
                .join(", "),
            self.weak_against
                .iter()
                .map(|t| format!("'{}'", t.name))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl DbDelete for PokemonType {}
