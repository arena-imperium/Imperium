use crate::game_ui::egui_wrappers::StrMap;
use crate::Scene;
use bevy::log;
use bevy::prelude::*;
use cuicui_chirp::ChirpBundle;

pub struct HangerScenePlugin;

impl Plugin for HangerScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, hanger.run_if(in_state(Scene::Hanger)));
        app.add_systems(OnEnter(Scene::Station), on_hanger_init);
        app.add_systems(OnExit(Scene::Station), on_hanger_exit);
    }
}
fn hanger(mut next_state: ResMut<NextState<Scene>>, mut init: Local<bool>) {
    if !*init {
        //next_state.set(Scene::Hanger);
        log::info!("Inside hanger");
        *init = true;
    }
}

#[derive(Default, Component)]
pub struct HangerUi;

// Setup the scene for when the station is focused on
pub fn on_hanger_init(
    mut cmds: Commands,
    serv: Res<AssetServer>,
    mut text_map: ResMut<StrMap>,
    camera_query: Query<Entity, With<Camera>>,
) {
    cmds.spawn((
        ChirpBundle::new(serv.load("ui/chirps/login_init.chirp")),
        HangerUi,
    ));
}

// Despawn scene
pub fn on_hanger_exit(mut cmds: Commands, ui: Query<Entity, With<HangerUi>>) {
    cmds.entity(ui.iter().next().unwrap()).despawn_recursive();
}
