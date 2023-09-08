use anchor_lang::prelude::*;

#[account()]
#[derive(Debug)]
pub struct SpaceShip {
    pub bump: u8,
    pub owner: Pubkey,
    pub hull: Hull,
}

impl SpaceShip {
    pub const LEN: usize = 8 + std::mem::size_of::<SpaceShip>();
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct Spaceship {
    pub hull: Hull,
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
