# List of stuff to be done (maybe use Node tools for it later on)

## Smart Contract

- handle passive powerups (in fight_engine.rs)

- make shield have 1 layer by default

- add passive power ups
        - afterburner (increase dodge chances)
        - Burst Projector (chance to jam either module)
        - EM smartbombs (missile that disable all drones for X turns)
        - shield booster: recharge a shield layer instantly
        - passive modules that increase stats:
                - armor plates : + 10 hull HP
                - capacitor battery: add a shield layer + reduce laser weapon charge time or smthg


- add new drones and mutatio to the LT
- add anti drone weapons
- add anti missiles drones
- add drone jammers

- look into macro (ask node again) to sideload loot table at compile time from a CSV

- test expired switchboard function request after 75 slots (warp and check it can be called again)

- implement fight engine logic (in arena_matchmaking_settle, currently placeholder)
        - One way is probably to have the docker create 2 fighter sheet, with all stats compiled form on chain accounts, and upload that temporarily onchain and pass it to the settle IX or smthg)

-- add a way to monitor anchor events

- Implement more shop items

- think about replayability of fight (check tg chat with Aleph.im)
