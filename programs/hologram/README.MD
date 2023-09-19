# Hologram Program (Smart Contract on Solana)

This smartcontract called Hologram is the Fully On Chain (FoC) game engine for the Imperium game.

## Lexicon

### Realm

A realm is similar terminologie to world of warcraft realms, an instance of the multiplayer game.

### UserAccount

Straightforward, contains the `Spaceship`s

### Spaceship

Equivalent to a RPG character. Progress in terms of experience and gear (power-ups: modules, drones, mutations)

### Power-ups

A RNG based reward on lvl-ing up spaceship.
There is a fixed loot table containing possible drops.
Possible drops are of 3 types: Modules, Drones and Mutations. A first roll with decide the drop type.
Drops are tier based, depending of spaceship level (lvl-ing up from lvl.2 to lvl.3 will have greater chance to drop a tier 1 power-up, and no chance to drop a tier-4 power up).
Inside Tiers, power-ups also have varying level of rarity.

## Technical

### Randomness

Randomness is mostly done through Xorshift function (implementation in utils/random_number_generator.rs).
Randomness initial seed is generated offchain using custom Switchboard Function.
<https://docs.switchboard.xyz/functions>

The deterministic nature of spaceship seeds provide a way to replay encounters easily knowing starting parameters, lowering the amount of data requiered to move back and forth between client and server.
In order to keep the game fair, some actions must then be un decidable by the player.
For instance picking an opponent must be decided server side, since both player have deterministic randomness.

Play
After an encounter, the player seed is updated using the opponent seed

`spaceship_seed_generation_function`: It serves for generating the initial seed of each spaceship.
Devnet: <https://app.switchboard.xyz/solana/devnet/user/A4PzGUimdCMv8xvT5gK2fxonXqMMayDm3eSXRvXZhjzU#2>
Mainnet:
Github: <https://github.com/arena-imperium/spaceship_seed_generation_function>

`arena_matchmaking_function`: It serves for matching two player without letting the player pick his opponent (since random is deterministic using xorshift)
Devnet: <https://app.switchboard.xyz/build/function/HQQC7a5KaVYS2ZK3oGohHqvTQqx4qZvbRxRVhEbz4sog>
Mainnet:
Github: <https://github.com/arena-imperium/arena_matchmaking_function>

Game design document : <https://www.notion.so/acammm/Imperium-GDD-09b55d9149f04f8b87c25b7d57ade3d8#91f408a6f189429d8632abdff4e1e4b9>
