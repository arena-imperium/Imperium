use crate::game_ui::dsl::{Mark, OnClick, UiAction};
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
    Transform, Update, Visibility, With,
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
    UiAction::add_action("login", || {
        // if this gets too big, split out into its own function
        OnClick::run(
            |text_map: Res<StrMap>,
             mut next_state: ResMut<NextState<crate::Scene>>,
             server: Res<HologramServer>| {
                let login_data = text_map.get("login_data").unwrap();
                // Todo: make actual solana login logic here
                //  And add extra states for waiting for login return val.
                //server.fire_fetch_account_task(&mut commands);

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
            |mut login_state: ResMut<LoginState>,
             mut sub_uis: Query<(Entity, &mut Visibility, &Mark)>| {
                log::info!("Closing window");
                // Switch which ui is visible
                for (_menu_entity, mut visibility, mark) in sub_uis.iter_mut() {
                    match mark.0.as_str() {
                        "init" => *visibility = Visibility::Inherited,
                        "window" => *visibility = Visibility::Hidden,
                        _ => {}
                    }
                }
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
    mut sub_uis: Query<(Entity, &mut Visibility, &Mark)>,
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

                // Go through our marked components, and change their visibility
                // to switch which ui is displayed.
                for (_menu_entity, mut visibility, mark) in sub_uis.iter_mut() {
                    match mark.0.as_str() {
                        "init" => *visibility = Visibility::Hidden,
                        "window" => *visibility = Visibility::Inherited,
                        _ => {}
                    }
                }
                *login_state = LoginState::LoginWindow;
            }
        }
        LoginState::LoginWindow => {
            /*if keyboard_input.just_pressed(KeyCode::Return) {

            }*/
        }
    }
}
