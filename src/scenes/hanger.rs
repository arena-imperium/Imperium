use crate::Scene;
use bevy::log;
use bevy::prelude::{in_state, App, IntoSystemConfigs, Local, NextState, Plugin, ResMut, Update};

pub struct HangerScenePlugin;

impl Plugin for HangerScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, hanger.run_if(in_state(Scene::Hanger)));
    }
}
fn hanger(mut next_state: ResMut<NextState<Scene>>, mut init: Local<bool>) {
    if !*init {
        //next_state.set(Scene::Hanger);
        log::info!("Inside hanger");
        *init = true;
    }
}
