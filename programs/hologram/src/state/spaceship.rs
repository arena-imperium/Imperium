use {
    super::{Experience, Fuel, SwitchboardFunctionRequestStatus, SwitchboardRequestInfo, Wallet},
    crate::{
        error::HologramError,
        utils::{LimitedString, RandomNumberGenerator},
        ARMOR_HITPOINTS_PER_ARMOR_LAYERING_LEVEL, BASE_ARMOR_HITPOINTS, BASE_CELERITY,
        BASE_DODGE_CHANCE, BASE_HULL_HITPOINTS, BASE_JAMMING_NULLIFYING_CHANCE,
        BASE_SHIELD_HITPOINTS, DODGE_CHANCE_CAP, DODGE_CHANCE_PER_MANOEUVERING_LEVEL_RATIO,
        FUEL_ALLOWANCE_AMOUNT, FUEL_ALLOWANCE_COOLDOWN, HULL_HITPOINTS_PER_LEVEL,
        JAMMING_NULLIFYING_CHANCE_CAP,
        JAMMING_NULLIFYING_CHANCE_PER_ELECTRONIC_SUBSYSTEMS_LEVEL_RATIO, MAX_LEVEL,
        MAX_POWERUP_SCORE, SHIELD_HITPOINTS_PER_SHIELD_SUBSYSTEMS_LEVEL,
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
    FactionOne,
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
        self.experience.available_stat_points > 0
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
        self.experience.credit_stat_point(1);
        // if the spaceship level is > 11, increase fuel capacity
        if self.experience.current_level > 11 {
            self.fuel.max += 1;
            self.fuel.refill(1)?
        }
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
    Launcher(WeaponModuleStats),
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
pub enum CycleTime {
    Short = 6,       // 5:3 (1.67)
    Accelerated = 8, // 5:4 (1.25)
    Standard = 10,   // 1:1 (1.0)
    Extended = 14,   // 5:7 (0.71)
    Long = 20,       // 1:2 (0.5)
    VeryLong = 30,   // 1:3 (0.33)
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum ProjectileSpeed {
    Sluggish = 40,
    Slow = 50,
    SubStandard = 60,
    Standard = 75,
    Fast = 90,
    Blazing = 105,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum Shots {
    Single,
    Salvo(u8),
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct WeaponModuleStats {
    pub class: AmmoClass,
    pub damage_profile: DamageProfile,
    pub shots: Shots,
    pub cycle_time: CycleTime,
    pub projectile_speed: ProjectileSpeed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct RepairModuleStats {
    pub repair_amount: u8,
    pub cycle_time: CycleTime,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub enum AmmoClass {
    Projectile,
    Missile,
    Energy,
}
