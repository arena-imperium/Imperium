use {
    crate::{
        error::HologramError,
        utils::{LimitedString, RandomNumberGenerator},
        ARMOR_HITPOINTS_PER_ARMOR_LAYERING_LEVEL, BASE_ARMOR_HITPOINTS, BASE_CELERITY,
        BASE_DODGE_CHANCE, BASE_HULL_HITPOINTS, BASE_JAMMING_NULLIFYING_CHANCE,
        BASE_SHIELD_HITPOINTS, DODGE_CHANCE_CAP, DODGE_CHANCE_PER_MANOEUVERING_LEVEL_RATIO,
        HULL_HITPOINTS_PER_LEVEL, JAMMING_NULLIFYING_CHANCE_CAP,
        JAMMING_NULLIFYING_CHANCE_PER_ELECTRONIC_SUBSYSTEMS_LEVEL_RATIO, MAX_LEVEL,
        MAX_XP_PER_LEVEL, SHIELD_HITPOINTS_PER_SHIELD_SUBSYSTEMS_LEVEL,
        SWITCHBOARD_FUNCTION_SLOT_UNTIL_EXPIRATION, XP_REQUIERED_PER_LEVEL_MULT,
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
    pub stats: Stats,
    // The resource used to join the Arena. Respenish daily.
    pub fuel: Fuel,
    pub experience: Experience,
    pub modules: Vec<Module>,
    pub drones: Vec<Drone>,
    pub mutations: Vec<Mutation>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct SpaceShipAnalytics {
    pub total_arena_matches: u16,
    pub total_arena_victories: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
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
    MythicalOne,
}

// All bonus described below are per level of the statistic. Players start with 0 level in all stats.
// Note: We do bonuses per X level in order to avoid floats (emulated on Solana eBPF and very costly in term of computing power)
// and also to avoid complicated things with basis points (BPS) and percentages calculations. KISS
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct Stats {
    // +20 Armor HP per level
    pub armor_layering: u8,
    // +20 Shield HP per level
    pub shield_subsystems: u8,
    // -1 charge_time projectile speed for all turrets type weapon per 5 levels
    // +1 projectile speed for all turrets type weapon per level
    pub turret_rigging: u8,
    // +1% Jamming Nullifying per 2 levels
    pub electronic_subsystems: u8,
    // +1% dodge per 2 levels
    // +1 Celerity
    pub manoeuvering: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct Fuel {
    pub max: u8,
    pub current: u8,
    // players can collect DAILY_FUEL_ALLOWANCE once per FUEL_ALLOWANCE_COOLDOWN period, this is the timestamp of their last collection
    pub daily_allowance_last_collection: i64,
}

impl Fuel {
    pub fn consume(&mut self, amount: u8) -> Result<()> {
        require!(self.current > amount, HologramError::InsufficientFuel);
        self.current -= amount;
        Ok(())
    }

    pub fn refill(&mut self, amount: u8) -> Result<()> {
        self.current = std::cmp::min(self.current + amount, self.max);
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct Experience {
    pub current_level: u8,
    pub current_exp: u16,
    pub exp_to_next_level: u16,
    pub available_stats_points: bool,
    pub available_crate: bool,
}

impl Experience {
    pub fn increase(&mut self, amount: u8) {
        self.current_exp += amount as u16;
        if self.current_exp >= self.exp_to_next_level {
            self.level_up();
        }
    }

    fn level_up(&mut self) {
        if self.current_level >= MAX_LEVEL {
            return;
        }
        self.current_level += 1;
        self.current_exp -= self.exp_to_next_level;
        self.exp_to_next_level = self.experience_to_next_level();
        self.available_stats_points = true;
        self.available_crate = true;
    }

    pub fn experience_to_next_level(&self) -> u16 {
        let xp_to_next_level = (self.current_level + 1) * XP_REQUIERED_PER_LEVEL_MULT;
        std::cmp::min(xp_to_next_level.into(), MAX_XP_PER_LEVEL)
    }
}

// Randomness is initially seeded using a Switchboard Function (custom).
// The function is called once only. Randomness is then iterated over using Xorshift.
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

impl SwitchboardRequestInfo {
    pub fn is_requested(&self) -> bool {
        match self.status {
            SwitchboardFunctionRequestStatus::Requested { slot: _ } => true,
            _ => false,
        }
    }

    pub fn request_is_expired(&self, current_slot: u64) -> bool {
        match self.status {
            SwitchboardFunctionRequestStatus::Requested {
                slot: requested_slot,
            } => {
                if current_slot > requested_slot + SWITCHBOARD_FUNCTION_SLOT_UNTIL_EXPIRATION as u64
                {
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
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
pub struct SwitchboardRequestInfo {
    pub account: Pubkey,
    pub status: SwitchboardFunctionRequestStatus,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum MatchMakingStatus {
    // the user is not in the queue
    None,
    // the user is queued and waiting for a match
    InQueue { slot: u64 },
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct Module {
    pub name: LimitedString,
    // pub description: LongLimitedString,
    pub rarity: Rarity,
    pub class: ModuleClass,
}
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct Drone {
    pub name: LimitedString,
    // pub description: LongLimitedString,
    pub rarity: Rarity,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct Mutation {
    pub name: LimitedString,
    // pub description: LongLimitedString,
    pub rarity: Rarity,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum SwitchboardFunctionRequestStatus {
    // No request has been made yet
    None,
    // The request has been made but the callback has not been called by the Function yet
    Requested { slot: u64 },
    // The request has been settled. the callback has been made
    Settled { slot: u64 },
    // The request has expired and was not settled
    Expired { slot: u64 },
}

// Ignore specific timestamp
impl PartialEq for SwitchboardFunctionRequestStatus {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SwitchboardFunctionRequestStatus::None, SwitchboardFunctionRequestStatus::None) => {
                true
            }
            (
                SwitchboardFunctionRequestStatus::Requested { slot: _ },
                SwitchboardFunctionRequestStatus::Requested { slot: _ },
            ) => true,
            (
                SwitchboardFunctionRequestStatus::Settled { slot: _ },
                SwitchboardFunctionRequestStatus::Settled { slot: _ },
            ) => true,
            _ => false,
        }
    }
}

impl SpaceShip {
    pub const LEN: usize = 8 + std::mem::size_of::<SpaceShip>();

    // the account has finished the initialization process
    pub fn is_initialized(&self) -> bool {
        match self.randomness.switchboard_request_info.status {
            SwitchboardFunctionRequestStatus::Settled { slot: _ } => true,
            _ => false,
        }
    }

    // informs wether the spaceship has available stat or crate point to spend.
    // Untils these are spent, he is barred from entering the arena
    pub fn has_no_pending_stats_or_crate(&self) -> bool {
        !(self.experience.available_stats_points || self.experience.available_crate)
    }

    // --- [Game engine code] ---

    // return the amount of experience a spaceship must earn to reach the next level
    // formula: next_level * XP_REQUIERED_PER_LEVEL_MULT, capped at MAX_XP_PER_LEVEL
    pub fn experience_to_next_level(&self) -> u16 {
        let xp_to_next_level = (self.experience.current_level + 1) * XP_REQUIERED_PER_LEVEL_MULT;
        std::cmp::min(xp_to_next_level.into(), MAX_XP_PER_LEVEL)
    }

    // return hull hitpoints for a ship - Value are prefight, they will be modified by the fight engine
    // formula: BASE_HULL_HITPOINTS + (current_level * HULL_HITPOINTS_PER_LEVEL)
    pub fn get_hull_hitpoints(&self) -> HitPoints {
        let hull_hitpoints =
            BASE_HULL_HITPOINTS + (self.experience.current_level as u16 * HULL_HITPOINTS_PER_LEVEL);
        HitPoints::init(hull_hitpoints)
    }

    // return armor hitpoints for a ship - Value are prefight, they will be modified by the fight engine
    // formula: BASE_ARMOR_HITPOINTS + (armor_layering * ARMOR_HITPOINTS_PER_ARMOR_LAYERING_LEVEL)
    pub fn get_armor_hitpoints(&self) -> HitPoints {
        let armor_hitpoints = BASE_ARMOR_HITPOINTS
            + self.stats.armor_layering as u16 * ARMOR_HITPOINTS_PER_ARMOR_LAYERING_LEVEL;
        HitPoints::init(armor_hitpoints)
    }

    // return shield hitpoints for a ship - Value are prefight, they will be modified by the fight engine
    // formula: BASE_SHIELD_HITPOINTS + (shield_subsystems * SHIELD_HITPOINTS_PER_SHIELD_SUBSYSTEMS_LEVEL)
    pub fn get_shield_hitpoints(&self) -> HitPoints {
        let shield_hitpoints = BASE_SHIELD_HITPOINTS
            + self.stats.shield_subsystems as u16 * SHIELD_HITPOINTS_PER_SHIELD_SUBSYSTEMS_LEVEL;
        HitPoints::init(shield_hitpoints)
    }

    // return turret_charge_time modifier for a ship - Value are prefight, they will be modified by the fight engine
    // formula: -1 charge_time per 5 levels
    pub fn get_turret_charge_time_reduction(&self) -> u8 {
        self.stats.turret_rigging / 5
    }

    // return turret_projectile_speed modifier for a ship - Value are prefight, they will be modified by the fight engine
    // formula: +1 projectile speed per turret_rigging stat level
    pub fn get_turret_projectile_speed(&self) -> u8 {
        self.stats.turret_rigging
    }

    // return ships current dodge chance value - Value may be modified by the fight engine
    // formula: (manoeuvering / DODGE_CHANCE_PER_MANOEUVERING_LEVEL_RATIO) + BASE_DODGE_CHANCE. Capped at DODGE_CHANCE_CAP
    pub fn get_dodge_chance(&self) -> u8 {
        std::cmp::min(
            (self.stats.manoeuvering / DODGE_CHANCE_PER_MANOEUVERING_LEVEL_RATIO)
                + BASE_DODGE_CHANCE,
            DODGE_CHANCE_CAP,
        )
    }

    // return ships current jamming nullifying chance value - Value may be modified by the fight engine
    // formula: (electronic_subsystems / JAMMING_NULLIFYING_CHANCE_PER_ELECTRONIC_SUBSYSTEMS_LEVEL_RATIO) + BASE_JAMMING_NULLIFYING_CHANCE. Capped at JAMMING_NULLIFYING_CHANCE_CAP
    pub fn get_jamming_nullifying_chance(&self) -> u8 {
        std::cmp::min(
            (self.stats.electronic_subsystems
                / JAMMING_NULLIFYING_CHANCE_PER_ELECTRONIC_SUBSYSTEMS_LEVEL_RATIO)
                + BASE_JAMMING_NULLIFYING_CHANCE,
            JAMMING_NULLIFYING_CHANCE_CAP,
        )
    }

    // return ships current celerity value - Value may be modified by the fight engine
    // formula: BASE_CELERITY + manoeuvering - number of modules, min 0
    pub fn get_celerity(&self) -> u8 {
        let celerity = BASE_CELERITY + self.stats.manoeuvering;
        let celerity_malus = self.modules.len() as u8;
        if celerity_malus > celerity {
            0
        } else {
            celerity - celerity_malus
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Mythical,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum TechTier {
    One,
    Two,
    Three,
    Four,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum HitpointLayer {
    Hull,
    Armor,
    Shield,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct HitPoints {
    pub current: u16,
    pub max: u16,
}

impl HitPoints {
    pub fn init(initial_value: u16) -> Self {
        HitPoints {
            current: initial_value,
            max: initial_value,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct DamageProfile {
    // 200% damage to shield, 25% damage to armor
    pub em: u8,
    // 200% damage to armor
    pub thermal: u8,
    // standard damage
    pub kinetic: u8,
    // 200% damage to hull, 25% damage to shield
    pub explosive: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum ModuleClass {
    // Weapons
    Turret(WeaponModuleStats),
    Exotic(WeaponModuleStats),
    // Repairers
    ShieldBooster(RepairModuleStats),
    ArmorRepairer(RepairModuleStats),
    HullRepairer(RepairModuleStats),
    // Passives
    ShieldAmplifier,
    NaniteCoating,
    TrackingComputer,
    Gyrostabilizer,
    // Jammers
    TrackingDisruptor,
    SensorDampener,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct WeaponModuleStats {
    pub class: WeaponClass,
    pub damage_profile: DamageProfile,
    pub shots: u8,
    pub charge_time: u8,
    pub projectile_speed: u8,
    // pub range: u8,
    // pub accuracy: u8,
    // pub tracking_speed: u8,
    // pub optimal_falloff: u8,
    // pub turret_size: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct RepairModuleStats {
    pub repair_amount: u8,
    pub cycle_time: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum WeaponClass {
    // turrets
    Energy,
    Projectile,
    Hybrid,
    // exotic
    EntropicDesintegrator,
    Missile,
}
