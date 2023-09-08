use {crate::utils::LimitedString, anchor_lang::prelude::*};

#[account()]
#[derive(Default)]
pub struct Realm {
    pub bump: u8,
    pub name: LimitedString,
    pub admin: Pubkey,
    pub stats: Stats,
}

impl Realm {
    pub const LEN: usize = 8 + std::mem::size_of::<Realm>();
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct Stats {
    pub total_user_accounts: u64,
    pub total_spaceships_created: u64,
}

impl Realm {
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
