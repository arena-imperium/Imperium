use crate::game_ui::egui_wrappers::StrMap;
use crate::Scene;
use bevy::log;
use bevy::prelude::*;
use cuicui_chirp::ChirpBundle;
use cuicui_dsl::dsl;
use cuicui_layout::dsl_functions::{child, pct};
use cuicui_layout_bevy_ui::UiDsl;

pub struct HangerScenePlugin;

impl Plugin for HangerScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, hanger.run_if(in_state(Scene::Hanger)));
        app.add_systems(OnEnter(Scene::Hanger), on_hanger_init);
        app.add_systems(OnExit(Scene::Hanger), on_hanger_exit);
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

#[derive(Default, Component)]
pub struct HangerSceneObj;

// Setup the scene for when the station is focused on
pub fn on_hanger_init(
    mut cmds: Commands,
    serv: Res<AssetServer>,
    mut text_map: ResMut<StrMap>,
    camera_query: Query<Entity, With<Camera>>,
) {
    log::info!("hanger init");
    cmds.spawn((
        ChirpBundle::new(serv.load("ui/chirps/hanger_menu.chirp")),
        HangerSceneObj,
    ));
    // Todo: figure out why macro based dsl ui doesn't show text.
    /*dsl! {
        <UiDsl>
        &mut cmds.spawn(HangerUi),
        Root(screen_root row distrib_start main_margin(50.0)) {
            Column(rules(pct(50), pct(50)) main_margin(10.0)){
                Column(column layout("vdSaS") rules(child(1.0), child(1.0)) main_margin(10.0) ) {
                    Text(font_size(30) text("TODO: Hanger UI"))
                }
            }
        }
    };*/
}

// Despawn scene
pub fn on_hanger_exit(
    mut cmds: Commands,
    ui: Query<Entity, With<HangerUi>>,
    hanger_scene: Query<Entity, With<HangerSceneObj>>,
) {
    cmds.entity(ui.iter().next().unwrap()).despawn_recursive();
    for entity in &hanger_scene {
        cmds.entity(entity).despawn();
    }
}
