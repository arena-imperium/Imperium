use anchor_lang::prelude::*;

#[account()]
#[derive(Debug)]
pub struct UserAccount {
    pub bump: u8,
    pub user: Pubkey,
    pub spaceships: Vec<Pubkey>,
}

impl UserAccount {
    pub const LEN: usize = 8 + std::mem::size_of::<UserAccount>();
}
