use {
    super::{Effect, PowerUp, PowerupKind},
    crate::{state::Bonuses, utils::LimitedString},
};

// Derived from a power up to be used in the fight engine
pub struct ConcretePowerup {
    pub name: LimitedString,
    // how much charge the module has accumulated
    pub accumulated_charge: u8,
    // how long it take to activate
    pub charge_time: u8,
    //
    pub accumulated_heat: u8,
    //
    pub heat: u8,
    // what the powerup does
    pub effect: Effect,
    pub bonuses: Option<Bonuses>,
    // the base type of the power up for filtering/ui purposes
    pub og_kind: PowerupKind,
    // Pointer to the original powerup
    pub og_powerup: Box<dyn PowerUp>,
}

impl ConcretePowerup {
    pub fn new(powerup: Box<dyn PowerUp>) -> Self {
        Self {
            name: powerup.get_name(),
            accumulated_charge: 0,
            charge_time: powerup.get_charge_time().unwrap_or(0),
            accumulated_heat: 0,
            heat: powerup.get_heat().unwrap_or(0),
            effect: powerup.get_effect(),
            bonuses: powerup.get_bonuses(),
            og_kind: powerup.get_kind(),
            og_powerup: powerup,
        }
    }

    pub fn is_active(&self) -> bool {
        self.og_powerup.is_active()
    }

    // Passives ----------------------------------------------------------------
    pub fn dissipate_heat(&mut self, amount: u8) {
        self.accumulated_heat = self.accumulated_heat.saturating_sub(amount);
    }

    pub fn is_off_cooldown(&self) -> bool {
        self.accumulated_heat == 0
    }

    pub fn heat(&mut self) {
        self.accumulated_heat = self.heat;
    }

    // Actives -----------------------------------------------------------------
    // charge the module and return true if it's been activated
    pub fn charge_and_activate(&mut self, amount: u8) -> bool {
        self.charge(amount);
        self.activate()
    }

    fn charge(&mut self, amount: u8) {
        self.accumulated_charge = self.accumulated_charge.saturating_add(amount);
    }

    fn activate(&mut self) -> bool {
        if self.is_charged() {
            self.accumulated_charge = self.accumulated_charge.saturating_sub(self.charge_time);
            return true;
        }
        return false;
    }

    fn is_charged(&self) -> bool {
        self.accumulated_charge >= self.charge_time
    }
}
