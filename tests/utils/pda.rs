use solana_sdk::pubkey::Pubkey;

pub fn get_realm_pda(name: &String) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"realm", name.as_bytes()], &hologram::id())
}

pub fn get_program_data_pda(program: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[program.as_ref()],
        &solana_program::bpf_loader_upgradeable::id(),
    )
}

pub fn get_user_account_pda(realm_pda: &Pubkey, user: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"user_account", realm_pda.as_ref(), user.as_ref()],
        &hologram::id(),
    )
}

pub fn get_spaceship_pda(
    realm_pda: &Pubkey,
    user: &Pubkey,
    spaceship_index: usize,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"spaceship",
            realm_pda.as_ref(),
            user.as_ref(),
            spaceship_index.to_le_bytes().as_ref(),
        ],
        &hologram::id(),
    )
}

pub fn get_switchboard_state_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[switchboard_solana::STATE_SEED],
        &switchboard_solana::SWITCHBOARD_ATTESTATION_PROGRAM_ID,
    )
}
