use crate::game_ui::dsl::{OnClick, UiAction};
use crate::game_ui::switch::SwitchToUI;
use crate::input_util::all_key_codes;
use crate::solana::{
    generate_test_client, HologramServer, SolanaFetchAccountTask, SolanaTransactionTask,
};
use crate::Scene;
use anchor_client::ClientError;
use bevy::asset::AssetServer;
use bevy::hierarchy::{BuildChildren, DespawnRecursiveExt};
use bevy::input::Input;
use bevy::log;
use bevy::prelude::{
    default, in_state, App, Camera, Commands, Component, Entity, EventWriter, IntoSystemConfigs,
    KeyCode, MouseButton, NextState, OnEnter, OnExit, Plugin, Query, Res, ResMut, Resource,
    SpriteBundle, Time, Transform, Update, With,
};
use cuicui_chirp::ChirpBundle;
use futures_lite::future::{block_on, poll_once};
use hologram::state::UserAccount;

/// Requires [`crate::GameGuiPlugin`]
pub struct StationScenePlugin;

impl Plugin for StationScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoginState>();
        app.add_systems(Update, station_login.run_if(in_state(Scene::Station)));
        app.add_systems(Update, station_move.run_if(in_state(Scene::Station)));
        app.add_systems(OnEnter(Scene::Station), on_station_init);
        app.add_systems(OnExit(Scene::Station), on_station_exit);
    }
}

#[derive(Default, Component)]
pub struct StationSceneObj;

// Setup the scene for when the station is focused on
pub fn on_station_init(
    mut cmds: Commands,
    serv: Res<AssetServer>,
    camera_query: Query<Entity, With<Camera>>,
) {
    // Need to add the actions before the ui.
    UiAction::add_action("connect_id", || {
        // if this gets too big, split out into its own function
        OnClick::run(
            |mut login_state: ResMut<LoginState>,
             mut next_state: ResMut<NextState<crate::Scene>>,
             server: Option<Res<HologramServer>>,
             mut event_writer: EventWriter<SwitchToUI>| {
                let mut if_no_server = || {
                    event_writer.send(SwitchToUI::new("select_wallet_window"));
                    *login_state = LoginState::SelectSolanaClientWindow
                };
                if let Some(server) = server {
                    if let Some(_account) = &server.user_account {
                        log::info!("Logging in, loading hangar");
                        // Todo: Play confirmation sound
                        // Transition directly to hangar.
                        next_state.set(Scene::Hangar)
                    } else {
                        if_no_server();
                    }
                } else {
                    if_no_server();
                }
            },
        )
    });

    UiAction::add_action("select_default_client", || {
        // if this gets too big, split out into its own function
        OnClick::run(
            |mut login_state: ResMut<LoginState>,
             mut commands: Commands,
             mut event_writer: EventWriter<SwitchToUI>| {
                // Also can consider caching the account pda
                let holo_server = HologramServer::new(generate_test_client());

                // Launch fetch account task to see if we need to tell the smart contract to
                // create a new account for this wallet or not
                holo_server.fire_fetch_account_task::<UserAccount>(
                    &mut commands,
                    &holo_server.calc_user_account_pda().0,
                );
                commands.insert_resource(holo_server);
                event_writer.send(SwitchToUI::new("loading"));
                *login_state = LoginState::CheckAccountExists;
                //next_state.set(Scene::Hangar)
            },
        )
    });

    UiAction::add_action("close", || {
        OnClick::run(
            |mut login_state: ResMut<LoginState>, mut event_writer: EventWriter<SwitchToUI>| {
                log::info!("Closing window");
                // Switch which ui is visible
                event_writer.send(SwitchToUI::new("init"));
                *login_state = LoginState::None;
            },
        )
    });

    cmds.spawn((
        ChirpBundle::new(serv.load("ui/chirps/station_ui.chirp")),
        StationSceneObj,
        StationUI,
    ));

    cmds.spawn((
        SpriteBundle {
            texture: serv.load("textures/bg_large.png"),
            transform: Transform::from_xyz(0.0, 0.0, -10.0),
            ..default()
        },
        StationSceneObj,
    ));
    // Spawn the station
    let station_entity = cmds
        .spawn((
            SpriteBundle {
                texture: serv.load("textures/station.png"),
                transform: Transform::from_xyz(0.0, 0.0, -1.0),
                ..default()
            },
            StationSceneObj,
            Station,
        ))
        .id();
    // Make the camera follow the station
    for camera_entity in camera_query.iter() {
        cmds.entity(camera_entity).set_parent(station_entity);
    }
}

// Despawn scene
pub fn on_station_exit(
    mut cmds: Commands,
    ui: Query<Entity, With<StationUI>>,
    station_scene: Query<Entity, With<StationSceneObj>>,
    tasks: Query<Entity, With<SolanaFetchAccountTask<UserAccount>>>,
) {
    cmds.entity(ui.iter().next().unwrap()).despawn_recursive();
    for entity in &station_scene {
        cmds.entity(entity).despawn();
    }
    for entity in &tasks {
        cmds.entity(entity).despawn();
    }
}

/// Station component. Currently we only have this.
/// In the future when multiple locations are used we can move this into a dedicated file.
/// or even module/directory with solar system login.
#[derive(Default, Component)]
pub struct Station;
#[derive(Default, Component)]
pub struct StationUI;

pub fn station_move(time: Res<Time>, mut station_query: Query<&mut Transform, With<Station>>) {
    let radius = 200.0;
    let rate = 0.008;

    for mut station_pos in station_query.iter_mut() {
        let angle = time.elapsed_seconds_f64() as f32 * rate;
        station_pos.translation.x = angle.cos() * radius;
        station_pos.translation.y = angle.sin() * radius;
    }
}

pub fn station_login(
    mut commands: Commands,
    mut login_state: ResMut<LoginState>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut event_writer: EventWriter<SwitchToUI>,
    mut server: Option<ResMut<HologramServer>>,
    mut fetch_acount: Query<(Entity, &mut SolanaFetchAccountTask<UserAccount>)>,
    create_account: Query<(Entity, &SolanaTransactionTask)>,
    mut next_state: ResMut<NextState<Scene>>,
) {
    match login_state.as_ref() {
        LoginState::None => {
            if keyboard_input.any_just_pressed(all_key_codes().into_iter().copied())
                || mouse_input.any_just_pressed([MouseButton::Right, MouseButton::Left])
            {
                // Could have a whole initialization sequence here
                // like
                // "sending request..."
                // "message recieved"
                // then interpolate/make the ui resize from zero in transition affect.

                event_writer.send(SwitchToUI::new("window"));
                *login_state = LoginState::LoginWindow;
            }
        }
        LoginState::LoginWindow => {
            /*if keyboard_input.just_pressed(KeyCode::Return) {

            }*/
        }
        LoginState::SelectSolanaClientWindow => {}
        LoginState::CheckAccountExists => {
            // Not sure about this closure; wanted to have the abort logic in one place
            // instead 5 places, to make sure nothings forgotten, but signature is long because
            // mutability issues and etc. Eh, probably fine to leave it as is at this point.
            let on_unrecoverable_error =
                |result: &dyn std::fmt::Debug,
                 login_state: &mut LoginState,
                 writer: &mut EventWriter<SwitchToUI>| {
                    writer.send(SwitchToUI::new("init"));
                    *login_state = LoginState::None;
                    log::error!("fetch task had unexpected error{result:?}");
                };
            for (entity, mut task) in fetch_acount.iter_mut() {
                // Handle all possible combinations of results for our fetch account task
                if let Some(result) = block_on(poll_once(&mut task.task)) {
                    match result {
                        Ok(account) => {
                            if let Some(server) = &mut server {
                                server.user_account = Some(account);
                                log::info!("successfully fetched user account");
                                // since we acquired all info needed in login, we can switch to the hangar scene
                                next_state.set(Scene::Hangar);
                            } else {
                                on_unrecoverable_error(
                                    &"expected server to be initialized",
                                    &mut login_state,
                                    &mut event_writer,
                                );
                            }
                        }
                        Err(error) => {
                            let error: ClientError = error;
                            match error {
                                ClientError::AccountNotFound => {
                                    if let Some(server) = &mut server {
                                        log::info!("account not found, creating one");
                                        event_writer.send(SwitchToUI::new("no_id"));
                                        *login_state = LoginState::CreateUserAccount;
                                        server.fire_default_create_user_account_task(&mut commands);
                                    } else {
                                        on_unrecoverable_error(
                                            &"expected server to be initialized",
                                            &mut login_state,
                                            &mut event_writer,
                                        );
                                    }
                                }
                                _ => on_unrecoverable_error(
                                    &error,
                                    &mut login_state,
                                    &mut event_writer,
                                ),
                            }
                        }
                    }
                    // Remove task entity since we are done handling its task now.
                    commands.entity(entity).despawn();
                } else {
                    log::trace!("waiting for fetch account response");
                    // Task is not yet complete.
                }
            }
        }
        LoginState::CreateUserAccount => {
            // solana.rs already has a built in system for handling creating accounts.
            // So what we do here is track it by measuring the number of create accounts
            // tasks in flight. Once there are no longer any in-flight, we can assume
            // the create account task completed.
            log::trace!("waiting for create account response");
            if create_account.iter().len() == 0 {
                log::info!("account should be created attempting to fetch it... ");
                if let Some(server) = server {
                    server.fire_fetch_account_task::<UserAccount>(
                        &mut commands,
                        &server.calc_user_account_pda().0,
                    );
                } else {
                    event_writer.send(SwitchToUI::new("init"));
                    *login_state = LoginState::None;
                    log::error!("in state LoginState::CreateUserAccount but hologram server wasn't initialized! aborting..");
                }
                // In this case we can expect the account to be created.
                // With it created, we switch back to the CheckAccountExists state
                // to handle acquiring it and continuing.
                event_writer.send(SwitchToUI::new("waiting"));
                *login_state = LoginState::CheckAccountExists;
            }
        }
    }
}

#[derive(Default, Resource)]
pub enum LoginState {
    #[default]
    None,
    LoginWindow,
    SelectSolanaClientWindow,
    CheckAccountExists,
    CreateUserAccount,
}
