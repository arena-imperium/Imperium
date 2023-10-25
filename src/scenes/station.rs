use crate::game_ui::dsl::{OnClick, UiAction};
use crate::game_ui::egui_wrappers::StrMap;
use crate::game_ui::LoginState;
use crate::input_util::all_key_codes;
use crate::solana::HologramServer;
use crate::Scene;
use bevy::asset::AssetServer;
use bevy::hierarchy::{BuildChildren, DespawnRecursiveExt};
use bevy::input::Input;
use bevy::log;
use bevy::prelude::{
    default, in_state, App, Camera, Commands, Component, Entity, IntoSystemConfigs, KeyCode,
    MouseButton, NextState, OnEnter, OnExit, Plugin, Query, Res, ResMut, SpriteBundle, Time,
    Transform, Update, With,
};
use cuicui_chirp::ChirpBundle;

/// Requires [`crate::GameGuiPlugin`]
pub struct StationScenePlugin;

impl Plugin for StationScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, station_login.run_if(in_state(Scene::Station)));
        app.add_systems(Update, station_move.run_if(in_state(Scene::Station)));
        app.add_systems(OnEnter(Scene::Station), on_station_init);
        app.add_systems(OnExit(Scene::Station), on_station_exit);
    }
}

#[derive(Default, Component)]
pub struct LoginInitUi;

// Setup the scene for when the station is focused on
pub fn on_station_init(
    mut cmds: Commands,
    serv: Res<AssetServer>,
    mut text_map: ResMut<StrMap>,
    camera_query: Query<Entity, With<Camera>>,
) {
    cmds.spawn((
        ChirpBundle::new(serv.load("ui/chirps/login_init.chirp")),
        LoginInitUi,
    ));

    cmds.spawn(SpriteBundle {
        texture: serv.load("textures/bg_large.png"),
        transform: Transform::from_xyz(0.0, 0.0, -10.0),
        ..default()
    });
    // Spawn the station
    let station_entity = cmds
        .spawn((
            SpriteBundle {
                texture: serv.load("textures/station.png"),
                transform: Transform::from_xyz(0.0, 0.0, -1.0),
                ..default()
            },
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
    ui: Query<Entity, With<LoginInitUi>>,
    station: Query<Entity, With<Station>>,
) {
    cmds.entity(ui.iter().next().unwrap()).despawn_recursive();
    cmds.entity(station.iter().next().unwrap())
        .despawn_recursive();
}

/// Station component. Currently we only have this.
/// In the future when multiple locations are used we can move this into a dedicated file.
/// or even module/directory with solar system login.
#[derive(Default, Component)]
pub struct Station;

pub fn station_move(
    time: Res<Time>,
    mut station_query: Query<&mut Transform, With<Station>>,
    text_map: Res<StrMap>,
) {
    let radius = 200.0;
    let rate = 0.008;

    for mut station_pos in station_query.iter_mut() {
        let angle = time.elapsed_seconds_f64() as f32 * rate;
        station_pos.translation.x = angle.cos() * radius;
        station_pos.translation.y = angle.sin() * radius;
    }
}

pub fn station_login(
    mut cmds: Commands,
    serv: Res<AssetServer>,
    mut next_state: ResMut<NextState<crate::Scene>>,
    mut login_state: ResMut<LoginState>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    ui: Query<Entity, With<LoginInitUi>>,
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

                // for now will simply change the ui to show the login ui.
                cmds.entity(ui.iter().next().unwrap()).despawn_recursive();
                cmds.spawn((
                    // Possibly proc gen ui elements from available wallets?
                    ChirpBundle::new(serv.load("ui/chirps/login_window.chirp")),
                    LoginInitUi,
                ));
                UiAction::add_action("login", || {
                    // if this gets too big, split out into its own function
                    OnClick::run(
                        |mut login_state: ResMut<LoginState>,
                         text_map: Res<StrMap>,
                         mut next_state: ResMut<NextState<crate::Scene>>,
                         server: Res<HologramServer>,
                         mut commands: Commands| {
                            let login_data = text_map.get("login_data").unwrap();
                            // Todo: make actual solana login logic here
                            //  And add extra states for waiting for login return val.
                            server.fire_fetch_account_task(&mut commands);

                            // For now we just directly consider any input as acceptable.
                            if login_data != "" {
                                log::info!("Logging in, loading hanger");
                                // Todo: Play confirmation sound
                                // Transition directly to hanger.
                                next_state.set(Scene::Hanger)
                            }
                            // We will count empty input as failure
                            else {
                                log::info!("Logging in failed!");
                                // Todo: Play error sound
                                // Make ui window shake or something?
                            }
                        },
                    )
                });
                UiAction::add_action("close", || {
                    OnClick::run(
                        |mut cmds: Commands,
                         mut login_state: ResMut<LoginState>,
                         text_map: Res<StrMap>,
                         ui: Query<Entity, With<LoginInitUi>>,
                         serv: Res<AssetServer>| {
                            log::info!("Closing window");
                            cmds.entity(ui.iter().next().unwrap()).despawn_recursive();
                            cmds.spawn((
                                // Possibly proc gen ui elements from available wallets?
                                ChirpBundle::new(serv.load("ui/chirps/login_init.chirp")),
                                LoginInitUi,
                            ));
                            *login_state = LoginState::None;
                        },
                    )
                });
                *login_state = LoginState::LoginWindow;
            }
        }
        LoginState::LoginWindow => {
            /*if keyboard_input.just_pressed(KeyCode::Return) {

            }*/
        }
    }
}
