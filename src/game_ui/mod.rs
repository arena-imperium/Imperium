mod dsl;
mod mirror;

use bevy::prelude::*;
use bevy::app::{App, Plugin, Update};
use bevy_mod_picking::{DefaultPickingPlugins, events};
use bevy_mod_picking::prelude::{On, Pointer};
use cuicui_chirp::ChirpBundle;
use cuicui_layout::{LayoutRootCamera};
use cuicui_layout_bevy_ui::{UiDsl as Dsl, UiDsl};
use crate::game_ui::dsl::{ImperiumDsl, OnClick, UiAction};
use crate::game_ui::mirror::MirrorPlugin;


/// Ie: what gamemode/scene are we currently in?
#[derive(Default, Clone, Eq, PartialEq, Debug, Hash, States)]
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
        app.add_plugins(cuicui_layout_bevy_ui::Plugin);
        app.add_plugins(cuicui_chirp::loader::Plugin::new::<ImperiumDsl>());
        app.add_plugins(DefaultPickingPlugins);
        app.add_plugins(MirrorPlugin::<OnClick, UiAction>::new_from());
        app.add_systems(Startup, setup);
        app.add_systems(Update, loading_screen.run_if(in_state(Scene::Loading)));
        app.add_systems(Update, station_login.run_if(in_state(Scene::Station)));
        app.add_systems(Update, loading_screen.run_if(in_state(Scene::Hanger)));
    }
}

fn setup(mut cmds: Commands, serv: Res<AssetServer>) {
    // Use LayoutRootCamera to mark a camera as the screen boundaries.
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 0.3;

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
