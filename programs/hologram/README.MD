# Hologram Program (Smart Contract on Solana)

This smartcontract called Hologram is the Fully On Chain (FoC) game engine for the Imperium game.

## Lexicon

### Realm

A realm is similar terminologie to world of warcraft realms, an instance of the multiplayer game.

### UserAccount

Straightforward, contains the `Spaceship`s

### Spaceship

Equivalent to a RPG character. Progress in terms of experience and gear (power-ups: modules, drones, mutations)

`spaceship_seed_generation_function`: It serves for generating the initial seed of each spaceship.
Devnet: <https://app.switchboard.xyz/build/function/5vPREeVxqBEyY499k9VuYf4A8cBVbNYBWbxoA5nwERhe>
Mainnet:
Github: <https://github.com/arena-imperium/spaceship_seed_generation_function>

`arena_matchmaking_function`: It serves for matching two player without letting the player pick his opponent (since random is deterministic using xorshift)
Devnet: <https://app.switchboard.xyz/build/function/HQQC7a5KaVYS2ZK3oGohHqvTQqx4qZvbRxRVhEbz4sog>
Mainnet:
Github: <https://github.com/arena-imperium/arena_matchmaking_function>

`crate_picking_function`: It seeds the crate drops
Devnet: <https://app.switchboard.xyz/build/function/EyAwVLdvBrrU2fyGsZbZEFArLBxT6j6zo59DByHF3AxG>
Mainnet:
Github: <https://github.com/arena-imperium/crate-picking-function>

Game design document : <https://www.notion.so/acammm/Imperium-GDD-09b55d9149f04f8b87c25b7d57ade3d8#91f408a6f189429d8632abdff4e1e4b9>
