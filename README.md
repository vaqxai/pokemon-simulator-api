![](https://github.com/vaqxai/pokemon-simulator/actions/workflows/rust.yml/badge.svg)

Docs available [here](https://vaqxai.github.io/pokemon-simulator-api/pokemon_simulator/index.html)

React frontend available [here](https://github.com/Daniel-K-Bracki/pokemon-simulator-frontend)

# Pokemon Simulator

A Pokemon battle simulator API server.

This crate provides a web server implementation for simulating Pokemon battles.
It uses Rocket framework for handling HTTP requests and implements CORS support
for cross-origin requests.

## Features

- RESTful API endpoints for Pokemon battle simulation
- CORS support for cross-origin requests
- JSON response formatting

## API Endpoints

- `GET /api/` - Health check endpoint that returns OK status
- `GET /api/pokemons` - A list of all pokemons
- `POST /api/pokemons` - With a pokemon JSON in the body (same format as what comes from the `GET /api/pokemons` endpoint) adds a new pokemon
- `GET /api/trainers` - A list of all trainers and their pokemon
- `POST /api/trainer_pokemons/<trainer_name>` - Creates a trainer with name `<trainer_name>`
- `DELETE /api/trainer_pokemons/<trainer_name>` - Deletes a trainer with name `<trainer_name>`
- `GET /api/trainer_pokemons/<trainer_name>` - Returns a full list of all pokemons of a particular trainer
- `POST /api/trainer_pokemons/<trainer_name>/<pokemon_name>` - Adds a pokemon to a trainer's team, name is case sensitive.
- `DELETE /api/trainer_pokemons/<trainer_name>/<pokemon_name>` - Removes a pokemon from a trainer's team, name is case sensitive.
- `GET /api/simulate_fight/<contender_name>/<challenger_name>` - Simulate a fight between two pokemon, names are case sensitive.
- `GET /api/simulate_trainer_fight/<challenger_name>/<challenger_strategy>/<contender_name>/<contender_strategy>` - Simulate a fight between two trainers, possible strategies listed below, names are case sensitive.

### Fight Strageies
The fight strategy changhes how a trainer picks their next pokemon upon a pokemon's faint, or the first pokemon to go and battle
- `StrongestAtk` - Always choose the pokemon with the highest attack stat in your team
- `StrongestDef` - Always choose the pokemon with the highest defense stat in your team
- `StrongestSum` - Always choose the pokemon that has the highest atk+def sum
- `StrongestType` - If you can, choose a pokemon that has the best type advantage over the current enemy pokemon, or, if not possible, use `StrongestSum` instead
- `Random` - Always choose a random pokemon

## Installation
### Prerequisites for Docker installation
- Docker
### Docker Compose installation
1. Copy the [`docker-compose.yml`](https://github.com/vaqxai/pokemon-simulator-api/blob/main/docker-compose.yml) file from the repository to your machine
2. 
    ```
    docker compose up -d
    ```
### Docker Installation
1.
    You can use the default `8000` port as `<api_port>` or choose your own. This is the port where the API will be listening for HTTP requests.
    ```
    docker run --name pokemons-backend --publish=<api_port>:8000 -d ghcr.io/vaqxai/pokemon-simulator-api:main
    ```

3. 
    ```
    docker run --name neo4j --publish=7474:7474 --publish=7687:7687 --volume=$HOME/neo4j/data:/data neo4j
    ```
4.
    ```
    docker network create pokemons-database
    docker network connect pokemons-database neo4j
    docker network connect pokemons-database pokemons-backend
    ```
5. 
    ```
    docker network inspect pokemons-database
    ```
6.  Write down neo4j's internal IP Address
7.
    ```
    docker cp pokemons-backend:/Config.toml Config.toml
    vi Config.toml (or use your preferred text editor), insert the correct internal docker IP, leave the default port unless you changed it
    docker cp Config.toml pokemons-backend:/config.toml
    docker restart pokemons-backend
    ```
### Prerequisites for manual installation
- An instance of `neo4j` database
- Rust nightly
### Manual installation
1.
    ```
    cargo build --release
    ```
2. Configure your database's connect info, copying the executable `target/release/pokemon-simulator`(`.exe`) and `Config.toml` from the project's root directory into the same folder, and editing `Config.toml`, rename it to `config.toml`
3. Run the generated executable 
## Pokemon Fight Algorithm
1. The pokemon with the highest `AGI`lity stat attacks first
2. The base damage is the pokemon's `ATK` (attack) stat
3. The type damage multiplier is calculated as follows, starting with a multiplier of `1`
    1. If the attacker's primary type is "Strong Against" the defender's primary type, add `0.375` to the type damage multiplier
    2. If the attacker's primary type is "Weak Against" the defender's primary type, subtract `0.225` from the type damage multiplier
    3. If the defender has a secondary type, and the attacker's primary type is "Strong Against" it, add `0.375` to the type damage multiplier
    4. If the defender has a secondary type, and the attacker's primary type is "Weak Against" it, subtract `0.225` from the type damage multiplier
    5. If the attacker has a secondary type, and the defender's primary type is "Weak Against" it, add `0.375` to the type damage multiplier
    6. If the attacker has a secondary type, and the defender's primary type is "Strong Against" it, subtract `0.225` from the type damage multiplier
    7. If both pokemon have a secondary type, and the defender's is "Weak Against" the attacker's, add `0.375` to the type damage multiplier
    8. If both pokemon have a secondary type, and the defender's is "Strong Against" the attacker's, subtract `0.225` from the type damage multiplier
       
4. The maximum type damage multiplier is `2.5`, the minimum is `0.1`. A type damage multiplier above `1.8` means an attack is "Super effective", while a type damage multiplier below `0.6` means an attack is "Not very effective"
5. A random multiplier between `0.8` and `1.2` is calculated
6. A defense multiplier is calculated by dividing the defender's `DEF`ense stat by the maximum value of `250.0`, multiplied by `0.75`, then subtracted from `1.0`, to give a total defense multiplier (which multiplies the damage incoming to the defender) between `0.0` for a `0 DEF` stat, and `0.75` for a `250 DEF` stat
7. The base damage is multiplied by the type damage multiplier, the random multiplier, and the defense multiplier.
8. The final damage is subtracted from the defender's `HP` (hit points) stat.
9. If the defender's `HP` falls below zero, a fight is concluded.
10. Otherwise, the roles of the attacker and the defender are reversed, the remaining `HP` is carried over to the next round, and the fight continues until one of the pokemons' `HP` falls to zero.

## Trainer Fight Algorithm
1. The trainer picked as the `contender` picks their pokemon first. If they've selected the `StrongestType` strategy, they use `StrongestSum` for their first pokemon instead (as the other party has yet to choose their pokemon)
2. The trainer picked as the `challenger` picks their pokemon according to their strategy.
3. The two pokemon fight using the regular Pokemon Fight Algorithm
4. The winner's remaining `HP` is carried over to the next round, and the party whose pokemon fainted picks a new one using their strategy.
5. The first party to run out of pokemon loses the battle.
