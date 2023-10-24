use bevy::app::{App, Plugin, Update};
use bevy::log;
use bevy::prelude::*;
use bevy::reflect::ReflectRef;
use bevy::render::render_resource::Texture;
use bevy_mod_picking::DefaultPickingPlugins;
use cuicui_chirp::{ChirpBundle, ChirpReader};
use cuicui_dsl::dsl;
use cuicui_layout::LayoutRootCamera;

use crate::game_ui::dsl::{ImperiumDsl, OnClick, UiAction};
use crate::game_ui::egui_wrappers::{CuiCuiEguiPlugin, StrMap};
use crate::game_ui::highlight::HighlightPlugin;
use crate::game_ui::mirror::MirrorPlugin;
use crate::input_util::all_key_codes;

mod dsl;
mod egui_wrappers;
mod highlight;
mod mirror;

/// Ie: what gamemode/scene are we currently in?
#[derive(Default, Clone, Eq, PartialEq, Debug, Hash, States, Copy)]
pub enum Scene {
    #[default]
    Loading,
    /// Starting scene, where the player can setup a connection with their wallet
    Station,
    /// Here the menu is drawn and waiting for player interaction
    Hanger,
}

#[derive(Default, Resource)]
pub enum LoginState {
    #[default]
    None,
    LoginWindow,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<Scene>();
        app.init_resource::<LoginState>();

        // Ui crates and functionality stuff
        app.add_plugins(cuicui_layout_bevy_ui::Plugin);
        app.add_plugins(cuicui_chirp::loader::Plugin::new::<ImperiumDsl>());
        app.add_plugins(DefaultPickingPlugins);
        app.add_plugins(MirrorPlugin::<OnClick, UiAction>::new_from());

        // custom ui modules
        app.add_plugins(HighlightPlugin);
        // Needed for text boxes and dynamic labels
        app.add_plugins(CuiCuiEguiPlugin);

        app.add_systems(Startup, setup);
        //app.add_systems(Startup, ui_test_scene.after(setup));

        app.add_systems(Update, loading_screen.run_if(in_state(Scene::Loading)));

        app.add_systems(Update, station_login.run_if(in_state(Scene::Station)));
        app.add_systems(Update, station_move.run_if(in_state(Scene::Station)));
        app.add_systems(OnEnter(Scene::Station), on_station_init);
        app.add_systems(OnExit(Scene::Station), on_station_exit);

        //app.add_systems(Update, hanger_scene.run_if(in_state(Scene::Hanger)));
    }
}

fn setup(mut cmds: Commands, serv: Res<AssetServer>, mut text_map: ResMut<StrMap>) {
    // Use LayoutRootCamera to mark a camera as the screen boundaries.
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 0.3;
    camera_bundle.transform.translation.x = 0.0;
    camera_bundle.transform.translation.y = 0.0;
    camera_bundle.transform.translation.z = 10.0;
    cmds.spawn((camera_bundle, LayoutRootCamera));
}
fn ui_test_scene(mut cmds: Commands, serv: Res<AssetServer>, mut text_map: ResMut<StrMap>) {
    UiAction::add_action("PrintHello", || {
        OnClick::run(
            // This is a system, you can pass in any bevy resources in the closure
            || log::info!("HI! you clicked a button! nice. now what.."),
        )
    });
    text_map.insert("my_label".to_owned(), "Label_contents_test".to_owned());
    text_map.insert("counter".to_owned(), "0".to_owned());

    UiAction::add_action("increment_counter", || {
        OnClick::run(|mut text_map: ResMut<StrMap>| {
            let string = text_map.get_mut("counter").unwrap();
            let mut num: i32 = string.parse().unwrap();
            num += 1;
            *string = num.to_string();
        })
    });

    cmds.spawn(ChirpBundle::new(serv.load("ui/chirps/test_menu.chirp")));
}

fn loading_screen(mut next_state: ResMut<NextState<Scene>>) {
    // Todo: when UI is decided on, draw bar showing loaded assets
    let loaded = true;
    if loaded {
        next_state.set(Scene::Station);
    }
}

#[derive(Default, Component)]
pub struct LoginInitUi;

// Setup the scene for when the station is focused on
fn on_station_init(
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
                // Todo: when proper texture is made, rescale.
                // or just get a smaller placeholder texture
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
fn on_station_exit(
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
fn station_move(
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

fn station_login(
    mut cmds: Commands,
    serv: Res<AssetServer>,
    mut next_state: ResMut<NextState<Scene>>,
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
                         mut next_state: ResMut<NextState<Scene>>| {
                            let login_data = text_map.get("login_data").unwrap();
                            // Todo: make actual solana login logic here
                            //  And add extra states for waiting for login return val.
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

fn hanger(mut next_state: ResMut<NextState<Scene>>) {
    //next_state.set(Scene::Hanger);
}
