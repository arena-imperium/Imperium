use bevy::prelude::*;
use bevy::app::{App, Plugin, Update};
use cuicui_layout::{dsl, LayoutRootCamera};
use cuicui_layout_bevy_ui::UiDsl as Dsl;


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
        app.add_systems(Startup, setup);
        app.add_systems(Update, loading_screen.run_if(in_state(Scene::Loading)));
        app.add_systems(Update, station_login.run_if(in_state(Scene::Station)));
        app.add_systems(Update, loading_screen.run_if(in_state(Scene::Hanger)));
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Use LayoutRootCamera to mark a camera as the screen boundaries.
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 0.3;
    commands.spawn((camera_bundle, LayoutRootCamera));

    dsl! { &mut commands.spawn_empty(),
        // Use screen_root to follow the screen's boundaries
        Entity(row screen_root) {
            Entity(row margin(9.) border(5, Color::CYAN) bg(Color::NAVY)) {
                Entity(ui("Hello world!"))
            }
        }
    };

    commands.spawn(SpriteBundle {
        texture: asset_server.load("textures/backgroundwithoutplanets.png"),
        ..default()
    });
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
