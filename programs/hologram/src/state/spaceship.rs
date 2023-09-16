use {crate::utils::LimitedString, anchor_lang::prelude::*};

#[account()]
#[derive(Debug)]
pub struct SpaceShip {
    pub bump: u8,
    pub owner: Pubkey,
    pub name: LimitedString,
    pub randomness: Randomness,
    pub hull: Hull,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct Randomness {
    pub switchboard_request: Pubkey,
    pub status: RandomnessStatus,
    pub original_seed: u64,
    pub current_seed: u64,
    pub iteration: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub enum RandomnessStatus {
    None = 0,
    Pending,
    Ready,
}

impl SpaceShip {
    pub const LEN: usize = 8 + std::mem::size_of::<SpaceShip>();

    pub fn is_ready(&self) -> bool {
        self.randomness.status == RandomnessStatus::Ready
    }
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
