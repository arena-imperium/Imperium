use bevy::app::{App, Plugin, Update};
use bevy::log;
use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
use cuicui_chirp::ChirpBundle;
use cuicui_layout::LayoutRootCamera;

use crate::game_ui::dsl::{ImperiumDsl, OnClick, UiAction};
use crate::game_ui::egui_wrappers::{CuiCuiEguiPlugin, StrMap};
use crate::game_ui::highlight::HighlightPlugin;
use crate::game_ui::mirror::MirrorPlugin;

mod dsl;
mod mirror;
mod highlight;
mod egui_wrappers;

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
pub struct LoginState(bool);

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
        app.add_systems(Update, loading_screen.run_if(in_state(Scene::Loading)));
        app.add_systems(Update, station_login.run_if(in_state(Scene::Station)));
        app.add_systems(Update, loading_screen.run_if(in_state(Scene::Hanger)));
    }
}

fn setup(mut cmds: Commands, serv: Res<AssetServer>, mut text_map: ResMut<StrMap>) {
    // Use LayoutRootCamera to mark a camera as the screen boundaries.
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 0.3;

    UiAction::add_action("PrintHello", || OnClick::run(
        // This is a system, you can pass in any bevy resources in the closure
        ||{
        log::info!("HI! you clicked a button! nice. now what..")
    }));
    text_map.insert("my_label".to_owned(), "Label_contents_test".to_owned());
    text_map.insert("counter".to_owned(), "0".to_owned());

    UiAction::add_action("increment_counter", || OnClick::run(
        |mut text_map: ResMut<StrMap>|{
            let string = text_map.get_mut("counter").unwrap();
            let mut num: i32 = string.parse().unwrap();
            num += 1;
            *string = num.to_string();
        }));

    cmds.spawn((camera_bundle, LayoutRootCamera));
    cmds.spawn(ChirpBundle::new(serv.load("ui/chirps/loading_menu.chirp")));

    //let dsl = UiDsl::default();

    /*cmds.spawn(SpriteBundle {
        texture: serv.load("textures/backgroundwithoutplanets.png"),
        ..default()
    });*/
}

fn loading_screen(mut next_state: ResMut<NextState<Scene>>,) {
    // Todo: when UI is decided on, draw bar showing loaded assets
    let loaded = true;
    if loaded {
        next_state.set(Scene::Station);
    }
}

fn station_login(mut next_state: ResMut<NextState<Scene>>, mut login_state: ResMut<LoginState>) {
    //next_state.set(Scene::Hanger);
}

fn hanger(mut next_state: ResMut<NextState<Scene>>,) {
    //next_state.set(Scene::Hanger);
}
