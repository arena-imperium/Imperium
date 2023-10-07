use {
    super::SpaceShip,
    crate::{error::HologramError, utils::LimitedString},
    anchor_lang::prelude::*,
};

#[account()]
#[derive(Default)]
pub struct Realm {
    pub bump: u8,
    pub name: LimitedString,
    pub admin: Pubkey, // must also be the owner of the Switchboard functions
    pub switchboard_info: SwitchboardInfo,
    // matchmaking queues for the arena (softcore). Each queue catters to a specific Ordnance range. Details in init_realm IX
    pub arena_matchmaking_queue: Vec<MatchmakingQueue>,
    pub analytics: RealmAnalytics,
}

impl Realm {
    pub const LEN: usize = 8 + std::mem::size_of::<Realm>();
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct SwitchboardInfo {
    pub authority: Pubkey,
    pub spaceship_seed_generation_function: Pubkey,
    pub arena_matchmaking_function: Pubkey,
    pub crate_picking_function: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct MatchmakingQueue {
    // maximum ordnance of the spaceships in the queue
    pub up_to_ordnance: u8,
    // up to ARENA_MATCHMAKING_SPACESHIPS_PER_RANGE spaceship can be in the queue.
    // After than when someone join a match is created selected a random spaceship from the queue.
    pub spaceships: [Option<Pubkey>; 5], // @HARDCODED ARENA_MATCHMAKING_SPACESHIPS_PER_RANGE anchor bug cannot use const here
    // since this will be modified concurently, we cannot process more request than there is spaceships in the queue
    pub matchmaking_request_count: u8,
}

impl MatchmakingQueue {
    // inform wether the queue is currently filled
    pub fn is_filled(&self) -> bool {
        self.spaceships.iter().all(Option::is_some)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct RealmAnalytics {
    pub total_user_accounts: u64,
    pub total_spaceships_created: u64,
    pub total_arena_matches: u64,
}

impl Realm {
    pub fn get_time() -> Result<i64> {
        let time = solana_program::sysvar::clock::Clock::get()?.unix_timestamp;
        if time > 0 {
            Ok(time)
        } else {
            Err(ProgramError::InvalidAccountData.into())
        }
    }

    pub fn get_slot() -> Result<u64> {
        let slot = solana_program::sysvar::clock::Clock::get()?.slot;
        if slot > 0 {
            Ok(slot)
        } else {
            Err(ProgramError::InvalidAccountData.into())
        }
    }

    // return the matchmaking queue matching the spaceship Ordnance
    pub fn get_matching_matchmaking_queue(
        &self,
        spaceship: &SpaceShip,
    ) -> Result<&MatchmakingQueue> {
        // find the queue matching spaceship Ordnance
        self.arena_matchmaking_queue
            .iter()
            .find(|q| q.up_to_ordnance >= spaceship.ordnance())
            .ok_or(error!(HologramError::MatchmakingQueueNotFound))
    }

    // return the matchmaking queue matching the spaceship Ordnance (mutable)
    pub fn get_matching_matchmaking_queue_mut(
        &mut self,
        spaceship: &SpaceShip,
    ) -> Result<&mut MatchmakingQueue> {
        // find the queue matching spaceship Ordnance
        self.arena_matchmaking_queue
            .iter_mut()
            .find(|q| q.up_to_ordnance >= spaceship.ordnance())
            .ok_or(error!(HologramError::MatchmakingQueueNotFound))
    }

    pub fn transfer_sol<'a>(
        source_account: AccountInfo<'a>,
        destination_account: AccountInfo<'a>,
        system_program: AccountInfo<'a>,
        amount: u64,
    ) -> Result<()> {
        let cpi_accounts = anchor_lang::system_program::Transfer {
            from: source_account,
            to: destination_account,
        };
        let cpi_context = anchor_lang::context::CpiContext::new(system_program, cpi_accounts);

        anchor_lang::system_program::transfer(cpi_context, amount)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn realloc<'a>(
        funding_account: AccountInfo<'a>,
        target_account: AccountInfo<'a>,
        system_program: AccountInfo<'a>,
        new_len: usize,
        zero_init: bool,
    ) -> Result<()> {
        let new_minimum_balance = Rent::get()?.minimum_balance(new_len);
        let lamports_diff = new_minimum_balance.saturating_sub(target_account.try_lamports()?);

        Realm::transfer_sol(
            funding_account,
            target_account.clone(),
            system_program,
            lamports_diff,
        )?;

        target_account
            .realloc(new_len, zero_init)
            .map_err(|_| ProgramError::InvalidRealloc.into())
    }
}
