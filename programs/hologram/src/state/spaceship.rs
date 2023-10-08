use {
    super::{Fuel, SwitchboardFunctionRequestStatus, SwitchboardRequestInfo, Wallet},
    crate::{
        error::HologramError,
        utils::{LimitedString, RandomNumberGenerator},
        FUEL_ALLOWANCE_AMOUNT, FUEL_ALLOWANCE_COOLDOWN, MAX_ORDNANCE,
    },
    anchor_lang::prelude::*,
};

#[account()]
#[derive(Debug)]
pub struct SpaceShip {
    pub bump: u8,
    pub owner: Pubkey,
    pub name: LimitedString,
    pub analytics: SpaceShipAnalytics,
    //
    pub randomness: Randomness,
    pub arena_matchmaking: ArenaMatchmaking,
    pub crate_picking: CratePicking,
    // The base skin of the Ship
    pub hull: Hull,
    // The resource used to join the Arena. Respenish daily.
    pub fuel: Fuel,
    pub wallet: Wallet,
    pub modules: Vec<Module>,
    pub drones: Vec<Drone>,
    pub mutations: Vec<Mutation>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct SpaceShipAnalytics {
    pub total_arena_matches: u16,
    pub total_arena_victories: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy)]
pub enum Hull {
    CommonOne,
    CommonTwo,
    CommonThree,
    UncommonOne,
    UncommonTwo,
    UncommonThree,
    UncommonFour,
    RareOne,
    RareTwo,
    FactionOne,
}

// Randomness is initially seeded using a Switchboard Function (custom).
// The function is called once only. Randomness is then iterated over using Xorshift.
// This is initially used for the Hull skin roll at spaceship creation, but it's available as an interative RNG for other use cases.
// github: https://github.com/acamill/spaceship_seed_generation_function
// devnet: https://app.switchboard.xyz/solana/devnet/function/5vPREeVxqBEyY499k9VuYf4A8cBVbNYBWbxoA5nwERhe
// mainet: @TODO
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct Randomness {
    pub switchboard_request_info: SwitchboardRequestInfo,
    // the first seed fetched from the switchboard request
    pub original_seed: u64,
    pub current_seed: u64,
    pub iteration: u64,
}

impl Randomness {
    pub fn advance_seed(&mut self) {
        let mut rng = RandomNumberGenerator::new(self.current_seed);
        self.current_seed = rng.next();
        self.iteration += 1;
    }
}

// Arena matchmaking is handled by a Switchboard Function (custom).
// github: https://github.com/acamill/@TODO
// devnet: https://app.switchboard.xyz/solana/devnet/function/@TODO
// mainet: @TODO
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ArenaMatchmaking {
    pub switchboard_request_info: SwitchboardRequestInfo,
    pub matchmaking_status: MatchMakingStatus,
}

// Crate picking is handled by a Switchboard Function (custom).
// github: https://github.com/acamill/@TODO
// devnet: https://app.switchboard.xyz/solana/devnet/function/@TODO
// mainet: @TODO
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct CratePicking {
    pub switchboard_request_info: SwitchboardRequestInfo,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum MatchMakingStatus {
    // the user is not in the queue
    None,
    // the user is queued and waiting for a match
    InQueue { slot: u64 },
    // the user is being matched
    Matching { slot: u64 },
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct Module {
    pub name: LimitedString,
    // pub description: LongLimitedString,
    pub rarity: Rarity,
    pub class: ModuleClass,
    pub is_active: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct Drone {
    pub name: LimitedString,
    // pub description: LongLimitedString,
    pub rarity: Rarity,
    pub size: DroneSize,
    pub class: DroneClass,
    pub is_active: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct Mutation {
    pub name: LimitedString,
    // pub description: LongLimitedString,
    pub rarity: Rarity,
    pub is_active: bool,
}

impl SpaceShip {
    pub const LEN: usize = 8 + std::mem::size_of::<SpaceShip>();

    // the account has finished the initialization process
    pub fn is_initialized(&self) -> bool {
        matches!(
            self.randomness.switchboard_request_info.status,
            SwitchboardFunctionRequestStatus::Settled { slot: _ }
        )
    }

    // the ordnance of the spaceship, a score loosely representing it's power
    pub fn ordnance(&self) -> u8 {
        self.modules.len() as u8 + self.drones.len() as u8 + self.mutations.len() as u8
    }

    pub fn fuel_allowance_is_available(&self, current_time: i64) -> Result<bool> {
        let cooldown = current_time
            .checked_sub(FUEL_ALLOWANCE_COOLDOWN)
            .ok_or(HologramError::Overflow)?;
        Ok(self.fuel.daily_allowance_last_collection < cooldown)
    }

    // refill fuel + update timestamp
    pub fn claim_fuel_allowance(&mut self, current_time: i64) -> Result<()> {
        self.fuel.refill(FUEL_ALLOWANCE_AMOUNT)?;
        self.fuel.daily_allowance_last_collection = current_time;
        Ok(())
    }

    pub fn mount_module(&mut self, module: Module) -> Result<()> {
        require!(
            self.ordnance() < MAX_ORDNANCE,
            HologramError::MaxOrdnanceReached
        );
        msg!("Module mounted: {:?}", module);
        self.modules.push(module);
        Ok(())
    }

    pub fn load_drone(&mut self, drone: Drone) -> Result<()> {
        require!(
            self.ordnance() < MAX_ORDNANCE,
            HologramError::MaxOrdnanceReached
        );
        msg!("Drone loaded: {:?}", drone);
        self.drones.push(drone);
        Ok(())
    }

    pub fn apply_mutation(&mut self, mutation: Mutation) -> Result<()> {
        require!(
            self.ordnance() < MAX_ORDNANCE,
            HologramError::MaxOrdnanceReached
        );
        msg!("Mutation applied: {:?}", mutation);
        self.mutations.push(mutation);
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Faction,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum HitpointLayer {
    Hull,
    Armor,
    Shield,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct HitPoints {
    pub current: u8,
    pub max: u8,
}

impl HitPoints {
    pub fn init(initial_value: u8) -> Self {
        HitPoints {
            current: initial_value,
            max: initial_value,
        }
    }

    pub fn increase_max(&mut self, amount: u8) {
        self.max = self.max.saturating_add(amount);
        self.current = self.current.saturating_add(amount);
    }

    pub fn depleted(&self) -> bool {
        self.current == 0
    }

    pub fn deplete(&mut self, amount: u8) {
        self.current = self.current.saturating_sub(amount);
    }

    pub fn resplenish(&mut self, amount: u8) {
        self.current = self.current.saturating_add(amount);
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum ModuleClass {
    Weapon(WeaponModuleStats),
    Repairer(Bonuses, RepairModuleStats),
    Capacitative(Bonuses, Passive),
}

impl PartialEq for ModuleClass {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum DroneClass {
    // Fighter
    Weapon(WeaponModuleStats),
    // Eletronic warfare drones
    ECM(JammerModuleStats),
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum DroneSize {
    Light,
    Medium,
    Heavy,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy)]
pub enum Shots {
    Single,
    Salvo(u8),
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct WeaponModuleStats {
    pub weapon_type: WeaponType,
    pub damage: u8,
    pub shots: Shots,
    pub charge_time: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy)]
pub enum RepairTarget {
    Hull,
    Shield,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct RepairModuleStats {
    pub target: RepairTarget,
    pub repair_amount: u8,
    pub charge_time: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct Bonuses {
    pub hull_hitpoints: u8,
    pub shield_layers: u8,
    pub dodge_chance: u8,
    pub jamming_nullifying_chance: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum Passive {
    // when the hull has taken a given amount of damage recentely (5 turns), it will heal a specific amount of HP
    CapacitativeRepair {
        threshold: u8,
        repair_amount: u8,
        target: RepairTarget,
    },
    ShieldRecharge {
        amount: u8,
    },
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct JammerModuleStats {
    pub charge_burn: u8,
    pub chance: u8,
    pub charge_time: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy, PartialEq)]
pub enum WeaponType {
    Projectile,
    Missile,
    Laser,
    Beam,
}
