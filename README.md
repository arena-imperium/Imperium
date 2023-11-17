# Imperium Solana

## Codebase Description

The codebase is divided into two main parts: the on-chain program (game server) located in `/programs/hologram/`, and a basic game client with menus for debug testing on devnet located in `/src/`.

The on-chain program handles the game logic and state, while the client is responsible for user interactions and displaying the game state.

The codebase also utilizes three Switchboard functions deployed on devnet for spaceship seed generation, arena matchmaking, and crate picking. These functions are integral to the game mechanics and are referenced throughout the codebase.

Environment variables are used to configure the Solana RPC and WebSocket URLs, as well as the payer key for transactions. These can be found and modified in the `.env.example` file.

For testing, the codebase supports both localnet testing using BanksClient and devnet testing. Instructions for both types of testing can be found further down in this README.

Future improvements and features to be implemented are tracked in the `TODO.md` file.

---

## High level game overview

Player create accounts on a given Realm, create spaceships (characters), and take part into fights to gain currencies and increase his gear.

---

## Hologram Solana Program Instructions

The Hologram Solana program is the on-chain component of the Imperium game. It handles the game logic and state.
The program contains several instructions for handling the game logic and state:

### initialize_realm

This instruction initializes a new realm. A realm can be thought of as an instance of the multiplayer game.

### create_user_account

This instruction creates a user account tied to a realm. This will store a player's information and spaceships.

### create_spaceship (and settlement)

This instruction creates a spaceship for a user account. A spaceship can be thought of as a character in an RPG. This call to the SB function `spaceship-seed-generation`.

### arena_matchmaking (and settlement)

This instruction queues a spaceship for matchmaking in the arena. This calls to the SB function `arena-matchmaking-function`.

### pick_crate (and settlement)

This instruction spend in game currency of a Spaceship to unlock a new power up based on RNG. This calls to the SB function `crate-picking-function`.

### claim_fuel_allowance

This instruction allows a player to claim free fuel for each of their spaceships once per FUEL_ALLOWANCE_COOLDOWN.

### Devnet switchboard functions

```rust
// DEVNET test spaceship seed generation function 5vPREeVxqBEyY499k9VuYf4A8cBVbNYBWbxoA5nwERhe
// DEVNET test arena matchmaking function 4fxj8rHfhhrE7gLLeo5w1Zt2TbiVeVDVAw7PgBC31VBL
// DEVNET test crate picking function EyAwVLdvBrrU2fyGsZbZEFArLBxT6j6zo59DByHF3AxG
// <https://app.switchboard.xyz/build/function/5vPREeVxqBEyY499k9VuYf4A8cBVbNYBWbxoA5nwERhe>
// <https://app.switchboard.xyz/build/function/4fxj8rHfhhrE7gLLeo5w1Zt2TbiVeVDVAw7PgBC31VBL>
// <https://app.switchboard.xyz/build/function/EyAwVLdvBrrU2fyGsZbZEFArLBxT6j6zo59DByHF3AxG>
```

### Documents

Game design document <https://www.notion.so/acammm/Imperium-GDD-09b55d9149f04f8b87c25b7d57ade3d8>

UI drafts <https://excalidraw.com/#room=f8fe65768aea434cea0a,qIfKqUO8zFjBdVw5sAhoZg>

---

## Testing

### BanksClient localnet testing

Will run instruction and omit Switchboard code (will directly call the settlement IX with mocked results)

`cargo test`

### Devnet testing

spaceship_seed_generation_function
<https://app.switchboard.xyz/solana/devnet/function/5vPREeVxqBEyY499k9VuYf4A8cBVbNYBWbxoA5nwERhe>
<https://github.com/acamill/spaceship_seed_generation_function>

arena_matchmaking_function
<https://app.switchboard.xyz/solana/devnet/function/TODO>
<https://github.com/acamill/arena_matchmaking_function>

Generate new keypair (to get a fresh program instance)
`solana-keygen new -o program_id_devnet.json --force`

Update the keypair in the program in Anchor.toml and lib.rs

Rebuild the program .so
`anchor build`

Deploy the program
`anchor deploy --provider.cluster devnet --program-keypair program_id_devnet.json --program-name hologram`

Run the client
`cargo run`

Upload IDL for having deserialized accounts on Solana Explorer
`anchor idl init <program_id> -f target/idl/hologram.json --provider.cluster devnet`
then
`anchor idl upgrade <program_id> -f target/idl/hologram.json --provider.cluster devnet`
