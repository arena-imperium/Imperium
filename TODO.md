# List of stuff to be done (maybe use Node tools for it later on)

## Smart Contract

- give a go at using same deps as solpg and compile to wasm <https://github.com/solana-playground/solana-playground>

- test pick_crate
- test allocate_stat_point
- test claim_fuel_allowance
- test expired switchboard function request after 75 slots (warp and check it can be called again)

- implement fight engine logic (in arena_matchmaking_settle, currently placeholder)
        - One way is probably to have the docker create 2 fighter sheet, with all stats compiled form on chain accounts, and upload that temporarily onchain and pass it to the settle IX or smthg)
- implement loot table/drop (pick_crate_settle, currently placeholder)

-- add a way to monitor anchor events

- think about replayability of fight (check tg chat with Aleph.im)

- add a draw mechanic for endless fights (both get 1 xp) (update doc too)

- how to host loot table, how to have it "updatable"

- IMPORTANT - switchboard function will need to enable the "authorized caller" and to be transfered to a PDA. Best is to init them from the program right away
