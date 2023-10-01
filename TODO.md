# List of stuff to be done (maybe use Node tools for it later on)

## Smart Contract

- add fuel capacity increase per lvl from 10-16
- rename BlakcMarket crate by BiomechanicalCache
- add currency wallet per spaceship
- tweak XP gains, loose don't get XP, only winner
- add currency gain, looser get half XP value, winner gets XP value
- change the pick_crate_IX to select the faction (and pay currency)

- add new drones and mutatio to the LT

- test claim_fuel_allowance

- in pick_crate_settle the spaceship should allocate more space to accomodate for the drop

- test expired switchboard function request after 75 slots (warp and check it can be called again)

- implement fight engine logic (in arena_matchmaking_settle, currently placeholder)
        - One way is probably to have the docker create 2 fighter sheet, with all stats compiled form on chain accounts, and upload that temporarily onchain and pass it to the settle IX or smthg)
- implement loot table/drop (pick_crate_settle, currently placeholder)

-- add a way to monitor anchor events

- think about replayability of fight (check tg chat with Aleph.im)

- add a draw mechanic for endless fights (both get 1 xp) (update doc too)

- how to host loot table, how to have it "updatable"

- IMPORTANT - switchboard function will need to enable the "authorized caller" and to be transfered to a PDA. Best is to init them from the program right away
   -- no need, we verify the signer is the secure enclave

- give a go at using same deps as solpg and compile to wasm <https://github.com/solana-playground/solana-playground>
