#[tokio::test]
async fn test_create_pokemon() {
    use crate::database::{link::DbLink, promise::Promised};
    use crate::pokemon::Pokemon;
    use crate::pokemon::ptype::{self, PokemonType};
    use crate::pokemon::stats::PokemonStats;

    let mut water = PokemonType::new_to_db("Water".to_string()).await.unwrap();

    let water_promise = water.as_promise();

    let mut electric = PokemonType::new_to_db("Electric".to_string())
        .await
        .unwrap();

    let electric_promise = electric.as_promise();

    let mut fire = PokemonType::new_to_db("Fire".to_string()).await.unwrap();

    let fire_promise = fire.as_promise();

    water
        .link_to(&electric_promise, &ptype::Relationship::StrongAgainst)
        .await
        .unwrap();
    water
        .link_to(&fire_promise, &ptype::Relationship::WeakAgainst)
        .await
        .unwrap();

    electric
        .link_to(&water_promise, &ptype::Relationship::WeakAgainst)
        .await
        .unwrap();
    electric
        .link_to(&fire_promise, &ptype::Relationship::StrongAgainst)
        .await
        .unwrap();

    fire.link_to(&water_promise, &ptype::Relationship::StrongAgainst)
        .await
        .unwrap();
    fire.link_to(&electric_promise, &ptype::Relationship::WeakAgainst)
        .await
        .unwrap();

    let _pikachu = Pokemon::new_to_db(
        "Pikachu".to_string(),
        electric.as_promise(),
        None,
        PokemonStats {
            hp: 35,
            attack: 55,
            defense: 40,
            agility: 90,
        },
    )
    .await
    .unwrap();
}
