use crate::game_ui::dsl::{OnClick, UiAction};
use crate::game_ui::egui_wrappers::StrMap;
use crate::game_ui::switch::SwitchToUI;
use crate::game_ui::LoginState;
use crate::input_util::all_key_codes;
use crate::solana::{generate_test_client, HologramServer};
use crate::Scene;
use bevy::asset::AssetServer;
use bevy::hierarchy::{BuildChildren, DespawnRecursiveExt};
use bevy::input::Input;
use bevy::log;
use bevy::prelude::{
    default, in_state, App, Camera, Commands, Component, Entity, EventWriter, IntoSystemConfigs,
    KeyCode, MouseButton, NextState, OnEnter, OnExit, Plugin, Query, Res, ResMut, SpriteBundle,
    Time, Transform, Update, With,
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
                if let Some(server) = server {
                    log::info!("Logging in, loading hanger");
                    // Todo: Play confirmation sound
                    // Transition directly to hanger.
                    next_state.set(Scene::Hanger)
                } else {
                    event_writer.send(SwitchToUI::new("sel_sol_window"));
                    *login_state = LoginState::SelectSolanaClientWindow
                    // Switch to client connection UI
                }
            },
        )
    });

    UiAction::add_action("select_default_client", || {
        // if this gets too big, split out into its own function
        OnClick::run(
            |mut login_state: ResMut<LoginState>,
             mut commands: Commands,
             mut next_state: ResMut<NextState<crate::Scene>>,
             server: Option<Res<HologramServer>>| {
                // Todo: Add window variant "loading" or something
                //  for when we wait for the web wallet or whatever to confirm
                commands.insert_resource(HologramServer::new(generate_test_client()));
                next_state.set(Scene::Hanger)
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
) {
    cmds.entity(ui.iter().next().unwrap()).despawn_recursive();
    for entity in &station_scene {
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
    mut login_state: ResMut<LoginState>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut event_writer: EventWriter<SwitchToUI>,
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
    }
}
