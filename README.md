# Imperium Solana

## BanksClient localnet testing

Will run instruction and omit Switchboard code (will directly call the settlement IX with mocked results)

`cargo test`

## Devnet testing

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
