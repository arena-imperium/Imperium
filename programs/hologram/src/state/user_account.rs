use {
    super::{Hull, SpaceShip},
    crate::utils::LimitedString,
    anchor_lang::prelude::*,
};

#[account()]
#[derive(Debug)]
pub struct UserAccount {
    pub bump: u8,
    pub user: Pubkey,
    pub spaceships: Vec<SpaceShipLite>,
}

// This is a subset of the SpaceShip account, mainly for the client to render the spaceship list
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct SpaceShipLite {
    pub name: LimitedString,
    pub hull: Hull,
    pub spaceship: Pubkey,
}

impl SpaceShipLite {
    pub fn from_spaceship_account<'info>(spaceship: &Account<'info, SpaceShip>) -> Self {
        Self {
            name: spaceship.name,
            hull: spaceship.hull,
            spaceship: spaceship.key(),
        }
    }
}

impl UserAccount {
    pub const LEN: usize = 8 + std::mem::size_of::<UserAccount>();
}
