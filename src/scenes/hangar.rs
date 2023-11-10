use crate::game_ui::dsl::{Mark, OnClick, UiAction};
use crate::game_ui::egui_wrappers::StrMap;
use crate::solana::{CreatedSpaceShip, HologramServer, SolanaFetchAccountTask};
use crate::Scene;
use bevy::log;
use bevy::prelude::*;
use cuicui_chirp::ChirpBundle;
use cuicui_dsl::dsl;
use cuicui_layout::dsl_functions::{child, pct};
use cuicui_layout_bevy_ui::UiDsl;
use futures_lite::future::{block_on, poll_once};
use hologram::state::{SpaceShip, UserAccount};
use solana_sdk::signature::Signer;
pub struct HangarScenePlugin;

impl Plugin for HangarScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_create_spaceships, refresh_ship_info)
                .run_if(in_state(Scene::Hangar))
                .before(hangar_loop),
        );
        app.add_systems(Update, hangar_loop.run_if(in_state(Scene::Hangar)));
        app.add_systems(OnEnter(Scene::Hangar), on_hangar_init);
        app.add_systems(OnExit(Scene::Hangar), on_hangar_exit);
    }
}

#[derive(Default, Component)]
pub struct HangarUi;

#[derive(Default, Component)]
pub struct HangarSceneObj;
#[derive(Default, Component)]
pub struct NewShipDialog;

// Setup the scene for when the station is focused on
pub fn on_hangar_init(
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
    server: Option<Res<HologramServer>>, // mut text_map: ResMut<StrMap>,
    mut next_state: ResMut<NextState<Scene>>,
    mut text_map: ResMut<StrMap>,
) {
    'outer: {
        if let Some(server) = server {
            if let Some(account) = &server.user_account {
                for ship in &account.spaceships {
                    server.fire_fetch_account_task::<SpaceShip>(&mut cmds, &ship.spaceship);
                }
                // if both server and server.user_account are cached
                // we can start the process to load ship data.
                break 'outer;
            }
        }
        // if not, something went wrong with sign in; we should never
        // get here, so go back to the station scene.
        next_state.set(Scene::Station);
    }

    UiAction::add_action("buy_spaceship", || {
        // if this gets too big, split out into its own function
        OnClick::run(|mut cmds: Commands, asset_server: Res<AssetServer>| {
            cmds.spawn((
                ChirpBundle::new(asset_server.load("ui/chirps/hangar_popup.chirp")),
                HangarSceneObj,
                NewShipDialog,
            ));
        })
    });

    UiAction::add_action("goto_station", || {
        OnClick::run(|mut next_state: ResMut<NextState<Scene>>| {
            next_state.set(Scene::Station);
        })
    });

    UiAction::add_action("cancel", || {
        OnClick::run(
            |mut cmds: Commands, popup: Query<Entity, With<NewShipDialog>>| {
                cmds.entity(popup.iter().next().unwrap())
                    .despawn_recursive();
            },
        )
    });

    text_map.insert("new_ship_name".to_owned(), "DefaultShip".to_owned());
    UiAction::add_action("confirm", || {
        OnClick::run(
            |mut cmds: Commands,
             popup: Query<Entity, With<NewShipDialog>>,
             server: Option<Res<HologramServer>>,
             mut text_map: ResMut<StrMap>| {
                server.as_ref().unwrap().fire_create_spaceship_task(
                    &mut cmds,
                    text_map.get("new_ship_name").unwrap(),
                    &server.as_ref().unwrap().calc_realm_pda().0,
                    // Todo: refactor solana.rs, change api to minimize need to pass in
                    //  manual key's like this, using data cachied in the
                    //  hologram server tyoe.
                    //  Also wrap the various PubKey types in Wrapper structs.
                    &server.as_ref().unwrap().solana_client.payer.pubkey(),
                );
                cmds.entity(popup.iter().next().unwrap())
                    .despawn_recursive();
            },
        )
    });

    log::info!("hangar init");
    cmds.spawn((
        ChirpBundle::new(asset_server.load("ui/chirps/hangar_menu.chirp")),
        HangarUi,
    ));
}

/// Handle the new spaceships, if we need to update state etc.
pub fn handle_create_spaceships(
    mut cmds: Commands,
    mut events: EventReader<CreatedSpaceShip>,
    server: Res<HologramServer>,
) {
    if !events.is_empty() {
        // If we created a new spaceship; fetch the updated account
        // to refresh the spaceship info
        server.fire_fetch_account_task::<UserAccount>(&mut cmds, &server.calc_user_account_pda().0);
        for event in events.iter() {
            println!("Spaceship creation success.");
        }
    }
}

pub fn refresh_ship_info(
    mut server: ResMut<HologramServer>,
    mut fetch_acount: Query<(Entity, &mut SolanaFetchAccountTask<UserAccount>)>,
    mut next_state: ResMut<NextState<Scene>>,
) {
    for (entity, mut task) in fetch_acount.iter_mut() {
        // Handle all possible combinations of results for our fetch account task
        if let Some(result) = block_on(poll_once(&mut task.task)) {
            match result {
                Ok(account) => {
                    server.user_account = Some(account);
                    log::info!("successfully refreshed user account info");
                    // since we acquired all info needed in login, we can switch to the hangar scene
                }
                Err(error) => {
                    next_state.set(Scene::Station);
                }
            }
        } else {
            log::trace!("waiting for fetch account response");
            // Task is not yet complete. Todo: put a loading animation here
            //  or something in the future
        }
    }
}

pub fn hangar_loop(
    mut cmds: Commands,
    mut query: Query<(&mut Visibility, &Mark)>,
    server: Res<HologramServer>,
) {
    if let Some(ship) = server
        .user_account
        .as_ref()
        .unwrap()
        .spaceships
        .iter()
        .next()
    {
        for (mut visibility, mark) in query.iter_mut() {
            if mark.0 == "ship_info_card" {
                *visibility = Visibility::Inherited;
            }
        }
    } else {
        for (mut visibility, mark) in query.iter_mut() {
            if mark.0 == "ship_info_card" {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

// Despawn scene
pub fn on_hangar_exit(
    mut cmds: Commands,
    ui: Query<Entity, With<HangarUi>>,
    hangar_scene: Query<Entity, With<HangarSceneObj>>,
) {
    cmds.entity(ui.iter().next().unwrap()).despawn_recursive();
    for entity in &hangar_scene {
        cmds.entity(entity).despawn();
    }
    UiAction::clear_actions();
}
