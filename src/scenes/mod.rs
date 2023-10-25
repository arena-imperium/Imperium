use crate::scenes::hanger::HangerScenePlugin;
use crate::scenes::station::StationScenePlugin;
use crate::Scene;
use bevy::prelude::{in_state, App, IntoSystemConfigs, NextState, Plugin, ResMut, Update};

mod hanger;
mod station;

/// Loads all the different scenes
pub struct ScenesPlugin;

impl Plugin for ScenesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StationScenePlugin);
        app.add_plugins(HangerScenePlugin);
        app.add_systems(Update, loading_screen.run_if(in_state(Scene::Loading)));

        //app.add_systems(Update, hanger_scene.run_if(in_state(Scene::Hanger)));
    }
}

fn loading_screen(mut next_state: ResMut<NextState<Scene>>) {
    // Todo: when UI is decided on, draw bar showing loaded assets
    let loaded = true;
    if loaded {
        next_state.set(Scene::Station);
    }
}
