# List of stuff to be done (maybe use Node tools for it later on)

## Smart Contract

- handle shield layer regen
        - in the battlecard, define each layer regen time
        - add an "active module" to each ship that for shield regeneration
        - add an effect "shield regen" in the ActiveEffects triggered by that module, it also update the regen time depending of the current layer status

- handle passive powerups (in fight_engine.rs)

- add new drones and mutatio to the LT
- add anti drone weapons
- add anti missiles drones
- add drone jammers

- look into macro (ask node again) to sideload loot table at compile time from a CSV

- test expired switchboard function request after 75 slots (warp and check it can be called again)

- implement fight engine logic (in arena_matchmaking_settle, currently placeholder)
        - One way is probably to have the docker create 2 fighter sheet, with all stats compiled form on chain accounts, and upload that temporarily onchain and pass it to the settle IX or smthg)

-- add a way to monitor anchor events

- think about replayability of fight (check tg chat with Aleph.im)

- add a draw mechanic for endless fights (both get 1 xp) (update doc too)
