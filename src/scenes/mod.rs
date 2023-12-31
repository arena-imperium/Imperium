use crate::scenes::hangar::HangarScenePlugin;
use crate::scenes::station::StationScenePlugin;
use crate::Scene;
use bevy::prelude::{in_state, App, IntoSystemConfigs, NextState, Plugin, ResMut, Update};

mod hangar;
mod station;

/// Loads all the different scenes
pub struct ScenesPlugin;

impl Plugin for ScenesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(StationScenePlugin);
        app.add_plugins(HangarScenePlugin);
        app.add_systems(Update, loading_screen.run_if(in_state(Scene::Loading)));

        //app.add_systems(Update, hangar_scene.run_if(in_state(Scene::Hangar)));
    }
}

fn loading_screen(mut next_state: ResMut<NextState<Scene>>) {
    // Todo: when Asset loading takes a while, draw bar showing loaded assets
    let loaded = true;
    if loaded {
        next_state.set(Scene::Station);
    }
}
