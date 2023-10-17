use {
    crate::{
        utils::{self, pda},
        IMPERIUM_AMF, SWITCHBOARD_ATTESTATION_QUEUE,
    },
    anchor_lang::ToAccountMetas,
    hologram::{
        instructions::{roll_opponent_spaceship, Faction},
        state::{MatchMakingStatus, Realm, SpaceShip, SwitchboardFunctionRequestStatus},
        utils::RandomNumberGenerator,
    },
    solana_program::pubkey::Pubkey,
    solana_program_test::{BanksClientError, ProgramTestContext},
    solana_sdk::signer::{keypair::Keypair, Signer},
    spl_associated_token_account::get_associated_token_address,
    spl_token::native_mint,
    std::str::FromStr,
    tokio::sync::RwLock,
};

pub async fn arena_matchmaking(
    program_test_ctx: &RwLock<ProgramTestContext>,
    user: &Keypair,
    realm_pda: &Pubkey,
    spaceship_index: u8,
    faction: Faction,
) -> std::result::Result<(), BanksClientError> {
    let (spaceship_pda, _) =
        utils::get_spaceship_pda(realm_pda, &user.pubkey(), spaceship_index);
    let spaceship_before = utils::get_account::<SpaceShip>(program_test_ctx, &spaceship_pda).await;
    let realm_before = utils::get_account::<Realm>(program_test_ctx, &realm_pda).await;
    let matchmaking_queue_before = realm_before
        .get_matching_matchmaking_queue(&spaceship_before)
        .unwrap();
    let spaceships_in_queue_before = matchmaking_queue_before
        .spaceships
        .iter()
        .filter_map(|x| x.as_ref())
        .count();

    // ==== WHEN ==============================================================
    let (user_account_pda, _) = pda::get_user_account_pda(&realm_pda, &user.pubkey());

    // Fetch the spaceship account
    let spaceship = utils::get_account::<SpaceShip>(program_test_ctx, &spaceship_pda).await;

    let switchboard_amf_request = spaceship.arena_matchmaking.switchboard_request_info.account;
    let (switchboard_state_pda, _) = utils::get_switchboard_state_pda();
    let switchboard_amf_request_escrow =
        get_associated_token_address(&switchboard_amf_request, &native_mint::ID);

    let accounts_meta = {
        let accounts = hologram::accounts::ArenaMatchmaking {
            user: user.pubkey(),
            realm: *realm_pda,
            user_account: user_account_pda,
            spaceship: spaceship_pda,
            switchboard_state: switchboard_state_pda,
            switchboard_attestation_queue: Pubkey::from_str(SWITCHBOARD_ATTESTATION_QUEUE).unwrap(),
            arena_matchmaking_function: Pubkey::from_str(IMPERIUM_AMF).unwrap(),
            switchboard_request: switchboard_amf_request,
            switchboard_request_escrow: switchboard_amf_request_escrow,
            system_program: solana_program::system_program::id(),
            token_program: switchboard_solana::anchor_spl::token::ID,
            switchboard_program: switchboard_solana::SWITCHBOARD_ATTESTATION_PROGRAM_ID,
        };

        let accounts_meta = accounts.to_account_metas(None);

        accounts_meta
    };

    utils::create_and_execute_hologram_ix(
        program_test_ctx,
        accounts_meta,
        hologram::instruction::ArenaMatchmaking {
            faction,
            spaceship_index,
        },
        Some(&user.pubkey()),
        &[user],
        None,
        None,
    )
    .await?;

    // ==== THEN ==============================================================
    let spaceship = utils::get_account::<SpaceShip>(program_test_ctx, &spaceship_pda).await;
    let realm = utils::get_account::<Realm>(program_test_ctx, &realm_pda).await;
    let matchmaking_queue = realm.get_matching_matchmaking_queue(&spaceship).unwrap();
    let spaceships_in_queue = matchmaking_queue
        .spaceships
        .iter()
        .filter_map(|x| x.as_ref())
        .count();

    // matchmaking_request_count updated
    if matchmaking_queue_before.is_filled() {
        assert_eq!(
            matchmaking_queue.matchmaking_request_count,
            matchmaking_queue_before.matchmaking_request_count + 1
        );

        // request status update
        assert!(matches!(
            spaceship.arena_matchmaking.switchboard_request_info.status,
            SwitchboardFunctionRequestStatus::Requested { slot: _ }
        ));
    } else {
        // or spaceship added to queue
        assert_eq!(spaceships_in_queue, spaceships_in_queue_before + 1);
    }

    // fuel cost updated
    assert_eq!(spaceship.fuel.current, spaceship_before.fuel.current - 1);

    // matchmaking status updated
    if matchmaking_queue_before.is_filled() {
        assert!(matches!(
            spaceship.arena_matchmaking.matchmaking_status,
            MatchMakingStatus::Matching { slot: _ }
        ));
    } else {
        assert!(matches!(
            spaceship.arena_matchmaking.matchmaking_status,
            MatchMakingStatus::InQueue { slot: _ }
        ));
    }

    // ==== AND ===============================================================
    // Because using the localnet/banksclient setup we cannot rely on switchboard function, we call the settlement directly
    // For matchmaking, this will be called ONLY when the queue was filled. This is when a fight happen
    if matchmaking_queue_before.is_filled() {
        let spaceship_before =
            utils::get_account::<SpaceShip>(program_test_ctx, &spaceship_pda).await;
        let realm_before = utils::get_account::<Realm>(program_test_ctx, &realm_pda).await;
        let matchmaking_queue_before = realm_before
            .get_matching_matchmaking_queue(&spaceship_before)
            .unwrap();
        let spaceships_in_queue_before = matchmaking_queue_before
            .spaceships
            .iter()
            .filter_map(|x| x.as_ref())
            .count();

        let mut opponents = vec![];
        for spaceship in &matchmaking_queue_before.spaceships {
            if let Some(spaceship) = spaceship {
                opponents.push(*spaceship);
            }
        }

        // ==== WHEN ==============================================================
        let enclave_signer = Keypair::new();
        let accounts_meta = {
            let accounts = hologram::accounts::ArenaMatchmakingSettle {
                enclave_signer: enclave_signer.pubkey(), // In the real world this is not called by anyone else than the docker container
                user: user.pubkey(),
                realm: *realm_pda,
                user_account: user_account_pda,
                spaceship: spaceship_pda,
                switchboard_request: switchboard_amf_request,
                arena_matchmaking_function: Pubkey::from_str(IMPERIUM_AMF).unwrap(),
                opponent_1_spaceship: opponents[0],
                opponent_2_spaceship: opponents[1],
                opponent_3_spaceship: opponents[2],
                opponent_4_spaceship: opponents[3],
                opponent_5_spaceship: opponents[4],
            };

            let accounts_meta = accounts.to_account_metas(None);

            accounts_meta
        };
        // The random generation is normally done on the switchboard side, but we do it here for the tests
        let generated_seed = rand::random::<u32>();

        utils::create_and_execute_hologram_ix(
            program_test_ctx,
            accounts_meta,
            hologram::instruction::ArenaMatchmakingSettle {
                generated_seed,
                faction,
            },
            Some(&user.pubkey()),
            &[&user, &enclave_signer],
            None,
            None,
        )
        .await?;

        // ==== THEN ==============================================================
        let spaceship = utils::get_account::<SpaceShip>(program_test_ctx, &spaceship_pda).await;
        let realm = utils::get_account::<Realm>(program_test_ctx, &realm_pda).await;
        let matchmaking_queue = realm.get_matching_matchmaking_queue(&spaceship).unwrap();
        let spaceships_in_queue = matchmaking_queue
            .spaceships
            .iter()
            .filter_map(|x| x.as_ref())
            .count();
        // the randomness is deterministic on chain, and can be reproduced here
        let mut rng = RandomNumberGenerator::new(generated_seed.into());
        let opponent_spaceship_pda =
            roll_opponent_spaceship(&mut rng, matchmaking_queue_before).unwrap();

        let opponent_spaceship =
            utils::get_account::<SpaceShip>(program_test_ctx, &opponent_spaceship_pda).await;

        // request status (for the caller spaceship only)
        assert!(matches!(
            spaceship.arena_matchmaking.switchboard_request_info.status,
            SwitchboardFunctionRequestStatus::Settled { slot: _ }
        ));

        // verify that opponent was removed from queue
        assert!(matchmaking_queue
            .spaceships
            .iter()
            .find(|s| **s == Some(opponent_spaceship_pda))
            .is_none());

        // redundant check spaceship removed from queue
        assert_eq!(spaceships_in_queue, spaceships_in_queue_before - 1);

        // matchmaking request count updated
        assert_eq!(
            matchmaking_queue.matchmaking_request_count,
            matchmaking_queue_before.matchmaking_request_count - 1
        );

        // XP distributed
        // Currencies distributed

        // matchmaking status updated (for both participants)
        assert!(matches!(
            spaceship.arena_matchmaking.matchmaking_status,
            MatchMakingStatus::None
        ));
        assert!(matches!(
            opponent_spaceship.arena_matchmaking.matchmaking_status,
            MatchMakingStatus::None
        ));
    }
    Ok(())
}
