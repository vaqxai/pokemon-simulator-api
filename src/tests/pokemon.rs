#[tokio::test]
async fn test_create_pokemon() {
    use crate::database::DbPut;
    use crate::pokemon::Pokemon;
    use crate::pokemon::PokemonStats;
    use crate::pokemon::PokemonType;

    let mut water = PokemonType {
        name: "Water".to_string(),
        strong_against: vec![],
        weak_against: vec![],
    };

    let mut electric = PokemonType {
        name: "Electric".to_string(),
        strong_against: vec![],
        weak_against: vec![],
    };

    let mut fire = PokemonType {
        name: "Fire".to_string(),
        strong_against: vec![],
        weak_against: vec![],
    };

    water.strong_against.push(fire.clone());
    water.weak_against.push(electric.clone());

    electric.strong_against.push(water.clone());
    electric.weak_against.push(fire.clone());

    fire.strong_against.push(electric.clone());
    fire.weak_against.push(water.clone());

    water.put_self().await.unwrap();
    electric.put_self().await.unwrap();
    fire.put_self().await.unwrap();

    let pikachu = Pokemon {
        name: "Pikachu".to_string(),
        primary_type: electric.clone(),
        secondary_type: None,
        stats: PokemonStats {
            hp: 35,
            attack: 55,
            defense: 40,
            agility: 90,
        },
    };

    pikachu.put_self().await.unwrap();
    pikachu.link_types_to_db().await.unwrap();

    /*
    // Delete the pokemons
    Pokemon::delete(&pikachu.name).await.unwrap();
    PokemonType::delete(&water.name).await.unwrap();
    PokemonType::delete(&electric.name).await.unwrap();
    PokemonType::delete(&fire.name).await.unwrap();

    // Check if the pokemon is deleted
    let pokemon = Pokemon::get_first(&pikachu.name).await;
    let fire = PokemonType::get_first(&fire.name).await;
    let electric = PokemonType::get_first(&electric.name).await;
    let water = PokemonType::get_first(&water.name).await;

    assert!(pokemon.is_err());
    assert!(fire.is_err());
    assert!(electric.is_err());
    assert!(water.is_err());
    */
}
