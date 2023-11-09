use crate::Scene;
use bevy::log;
use bevy::prelude::*;
use cuicui_chirp::ChirpBundle;
/*
use cuicui_dsl::dsl;
use cuicui_layout::dsl_functions::{child, pct};
use cuicui_layout_bevy_ui::UiDsl;
*/
pub struct HangarScenePlugin;

impl Plugin for HangarScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, hangar.run_if(in_state(Scene::Hangar)));
        app.add_systems(OnEnter(Scene::Hangar), on_hangar_init);
        app.add_systems(OnExit(Scene::Hangar), on_hangar_exit);
    }
}
fn hangar(mut init: Local<bool>) {
    if !*init {
        log::info!("Inside hangar");
        *init = true;
    }
}

#[derive(Default, Component)]
pub struct HangarUi;

#[derive(Default, Component)]
pub struct HangarSceneObj;

// Setup the scene for when the station is focused on
pub fn on_hangar_init(
    mut cmds: Commands,
    serv: Res<AssetServer>,
    // mut text_map: ResMut<StrMap>,
) {
    log::info!("hangar init");
    cmds.spawn((
        ChirpBundle::new(serv.load("ui/chirps/hangar_menu.chirp")),
        HangarSceneObj,
    ));
    // Todo: figure out why macro based dsl ui doesn't show text.
    /*dsl! {
        <UiDsl>
        &mut cmds.spawn(HangarUi),
        Root(screen_root row distrib_start main_margin(50.0)) {
            Column(rules(pct(50), pct(50)) main_margin(10.0)){
                Column(column layout("vdSaS") rules(child(1.0), child(1.0)) main_margin(10.0) ) {
                    Text(font_size(30) text("TODO: Hangar UI"))
                }
            }
        }
    };*/
}

// Despawn scene
pub fn on_hangar_exit(
    mut cmds: Commands,
    ui: Query<Entity, With<HangarUi>>,
    hangar_scene: Query<Entity, With<HangarSceneObj>>,
) {
    cmds.entity(ui.iter().next().unwrap()).despawn_recursive();
    for entity in &hangar_scene {
        cmds.entity(entity).despawn();
    }
}
