![](https://github.com/vaqxai/pokemon-simulator/actions/workflows/rust.yml/badge.svg)

Docs available [here](https://vaqxai.github.io/pokemon-simulator/pokemon_simulator/index.html)

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
