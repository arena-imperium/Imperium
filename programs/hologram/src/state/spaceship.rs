use {
    super::{Experience, Fuel, SwitchboardFunctionRequestStatus, SwitchboardRequestInfo, Wallet},
    crate::{
        error::HologramError,
        utils::{LimitedString, RandomNumberGenerator},
        BASE_DODGE_CHANCE, BASE_HULL_HITPOINTS, BASE_JAMMING_NULLIFYING_CHANCE, BASE_SHIELD_LAYERS,
        DODGE_CHANCE_CAP, DODGE_CHANCE_PER_MANOEUVERING_LEVEL_RATIO, FUEL_ALLOWANCE_AMOUNT,
        FUEL_ALLOWANCE_COOLDOWN, HULL_HITPOINTS_PER_LEVEL, JAMMING_NULLIFYING_CHANCE_CAP,
        JAMMING_NULLIFYING_CHANCE_PER_WEAPON_RIGGING_LEVEL_RATIO, MAX_LEVEL, MAX_POWERUP_SCORE,
        SHIELD_LAYER_PER_SHIELD_LEVEL,
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
    // The statistics of the Ship (RPG like, like Strengh, Agility, etc.)
    pub subsystems: Subsystems,
    // The resource used to join the Arena. Respenish daily.
    pub fuel: Fuel,
    pub experience: Experience,
    // The "gear level" of the ship. Max of MAX_POWERUP_SCORE
    pub powerup_score: u8,
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

// All bonus described below are per level of the statistic. Players start with 0 level in all stats.
// Note: We do bonuses per X level in order to avoid floats (emulated on Solana eBPF and very costly in term of computing power)
// and also to avoid complicated things with basis points (BPS) and percentages calculations. KISS
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct Subsystems {
    // +1 shield layer per 2 levels (max 8 levels)
    pub shield: u8,
    // +2 Hull HP per level (max 10 levels)
    pub hull_integrity: u8,
    // +1% Jamming Nullifying per level
    pub weapon_rigging: u8,
    // +1% dodge per 2 levels
    pub manoeuvering: u8,
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

    // informs wether the spaceship has available stat or crate point to spend.
    // Untils these are spent, he is barred from entering the arena
    pub fn has_available_stat_point(&self) -> bool {
        self.experience.available_subsystem_upgrade_points > 0
    }

    pub fn gain_experience(&mut self, amount: u8) -> Result<()> {
        self.experience.current_exp += amount as u16;
        if self.experience.current_exp >= self.experience.exp_to_next_level {
            self.level_up()?
        }
        Ok(())
    }

    fn level_up(&mut self) -> Result<()> {
        if self.experience.current_level >= MAX_LEVEL {
            msg!("Max level reached, no more XP can be gained");
            return Ok(());
        }
        // increase level
        self.experience.current_level += 1;
        // remove XP points that were used to level up
        self.experience.current_exp -= self.experience.exp_to_next_level;
        // update xp to next level
        self.experience.exp_to_next_level = self.experience.experience_to_next_level();
        // give a stat point
        self.experience.credit_subsystem_upgrade_point(1);
        Ok(())
    }

    pub fn mount_module(&mut self, module: Module) -> Result<()> {
        require!(
            self.powerup_score < MAX_POWERUP_SCORE,
            HologramError::MaxPowerupScoreReached
        );
        msg!("Module mounted: {:?}", module);
        self.modules.push(module);
        self.powerup_score += 1;
        Ok(())
    }

    pub fn load_drone(&mut self, drone: Drone) -> Result<()> {
        require!(
            self.powerup_score < MAX_POWERUP_SCORE,
            HologramError::MaxPowerupScoreReached
        );
        msg!("Drone loaded: {:?}", drone);
        self.drones.push(drone);
        self.powerup_score += 1;
        Ok(())
    }

    pub fn apply_mutation(&mut self, mutation: Mutation) -> Result<()> {
        require!(
            self.powerup_score < MAX_POWERUP_SCORE,
            HologramError::MaxPowerupScoreReached
        );
        msg!("Mutation applied: {:?}", mutation);
        self.mutations.push(mutation);
        self.powerup_score += 1;
        Ok(())
    }

    // --- [Game engine code] ---

    // return hull hitpoints for a ship - Value are prefight, they will be modified by the fight engine
    // formula: BASE_HULL_HITPOINTS + (current_level * HULL_HITPOINTS_PER_LEVEL)
    pub fn get_hull_hitpoints(&self) -> HitPoints {
        let hull_hitpoints =
            BASE_HULL_HITPOINTS + (self.experience.current_level * HULL_HITPOINTS_PER_LEVEL);
        HitPoints::init(hull_hitpoints)
    }

    // the number of shield layers of the ship
    pub fn get_shield_layers(&self) -> HitPoints {
        let shield_layers =
            BASE_SHIELD_LAYERS + self.subsystems.shield * SHIELD_LAYER_PER_SHIELD_LEVEL;
        HitPoints::init(shield_layers)
    }

    // return ships current dodge chance value - Value may be modified by the fight engine
    // formula: (manoeuvering / DODGE_CHANCE_PER_MANOEUVERING_LEVEL_RATIO) + BASE_DODGE_CHANCE. Capped at DODGE_CHANCE_CAP
    pub fn get_dodge_chance(&self) -> u8 {
        std::cmp::min(
            (self.subsystems.manoeuvering / DODGE_CHANCE_PER_MANOEUVERING_LEVEL_RATIO)
                + BASE_DODGE_CHANCE,
            DODGE_CHANCE_CAP,
        )
    }

    // return ships current jamming nullifying chance value - Value may be modified by the fight engine
    // formula: (electronic_subsystems / JAMMING_NULLIFYING_CHANCE_PER_ELECTRONIC_SUBSYSTEMS_LEVEL_RATIO) + BASE_JAMMING_NULLIFYING_CHANCE. Capped at JAMMING_NULLIFYING_CHANCE_CAP
    pub fn get_jamming_nullifying_chance(&self) -> u8 {
        std::cmp::min(
            (self.subsystems.weapon_rigging
                / JAMMING_NULLIFYING_CHANCE_PER_WEAPON_RIGGING_LEVEL_RATIO)
                + BASE_JAMMING_NULLIFYING_CHANCE,
            JAMMING_NULLIFYING_CHANCE_CAP,
        )
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

    pub fn depleted(&self) -> bool {
        self.current == 0
    }

    pub fn deplete(&mut self, amount: u8) {
        self.current.saturating_sub(amount);
    }

    pub fn resplenish(&mut self, amount: u8) {
        self.current.saturating_add(amount);
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum ModuleClass {
    // Weapons
    Turret(WeaponModuleStats),
    Exotic(WeaponModuleStats),
    // Repairers
    ShieldBooster(RepairModuleStats), // provide a boost of power that instantly regenerate shield layer
    HullRepairer(RepairModuleStats),
    // Passives
    ShieldAmplifier,  // reduce shield layer regeneration time
    TrackingComputer, // reduce opponent dodge chances
}

impl PartialEq for ModuleClass {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum DroneClass {
    // Fighter
    Turret(WeaponModuleStats),
    Exotic(WeaponModuleStats),
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

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct RepairModuleStats {
    pub repair_amount: u8,
    pub charge_time: u8,
}

// Jamming works by randomly picking a module/drone, and attempting to jam it, which remove charge time from it
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct JammerModuleStats {
    pub charge_burn: u8,
    pub chance: u8,
    pub charge_time: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub enum WeaponType {
    Projectile,
    Missile,
    Laser,
    Beam,
}
