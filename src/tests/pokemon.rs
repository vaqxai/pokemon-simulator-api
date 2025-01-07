#[tokio::test]
async fn test_create_pokemon() {
    use crate::database::DbLink;
    use crate::pokemon::Pokemon;
    use crate::pokemon::PokemonStats;
    use crate::pokemon::PokemonType;
    use crate::pokemon::PokemonTypeRelationship;

    let mut water = PokemonType::new_to_db("Water".to_string()).await.unwrap();
    let mut electric = PokemonType::new_to_db("Electric".to_string())
        .await
        .unwrap();
    let mut fire = PokemonType::new_to_db("Fire".to_string()).await.unwrap();

    water
        .link_to(&electric, &PokemonTypeRelationship::StrongAgainst)
        .await
        .unwrap();
    water
        .link_to(&fire, &PokemonTypeRelationship::WeakAgainst)
        .await
        .unwrap();

    electric
        .link_to(&water, &PokemonTypeRelationship::WeakAgainst)
        .await
        .unwrap();
    electric
        .link_to(&fire, &PokemonTypeRelationship::StrongAgainst)
        .await
        .unwrap();

    fire.link_to(&water, &PokemonTypeRelationship::StrongAgainst)
        .await
        .unwrap();
    fire.link_to(&electric, &PokemonTypeRelationship::WeakAgainst)
        .await
        .unwrap();

    let _pikachu = Pokemon::new_to_db(
        "Pikachu".to_string(),
        electric.clone(),
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
