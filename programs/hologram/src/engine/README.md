# Game engine informations

## Match resolution

The `fight()` method in `fight_engine.rs` is where the match is resolved.
The whole fight is based on the provided `fight_seed` and can then be replayed using the `fight()` method offchain.

Steps:

- the spaceship and its opponent are converted to `SpaceshipBattleCard`s, a convenient data representation that only expose what the model needs to know.
- the starting spaceship is picked based on its `celerity` attribute (higher the better)
- a max of 100 turns now start. each of them is composed of:
  - checking if that player `is_defeated()` (hull HP == 0)
  - charge active powerups and trigger the one that have reached full charge (collecting them for later processing)
  - resolve effects of triggered powerups
  - increment turn by 1
  - update next turn spaceship

### Dodge

Calculated in battlecard `fire_at()` method.
A dodge roll is done first thing:

- if the attack is `Shots::Single` it will miss entierly
- if the attack is `Shots::Salvo(x)` only half (floor) will hit

### Crits

Crit is only possible on single shot attacks, base chance is 5% increased by projectile speed

### Single vs Salvo

Salvo are never entierly dodged, while single shot can crit
