use bevy::app::{App, Plugin};
use bevy::log;
use bevy::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
use cuicui_chirp::ChirpBundle;
use cuicui_layout::LayoutRootCamera;

use crate::game_ui::dsl::{ImperiumDsl, OnClick, UiAction};
use crate::game_ui::egui_wrappers::{CuiCuiEguiPlugin, StrMap};
use crate::game_ui::highlight::HighlightPlugin;
use crate::game_ui::mirror::MirrorPlugin;
use crate::game_ui::switch::SwitchPlugin;

pub mod dsl;
pub mod egui_wrappers;
mod highlight;
mod mirror;
pub mod switch;

pub struct GameGuiPlugin;

impl Plugin for GameGuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<crate::Scene>();

        // Ui crates and functionality stuff
        app.add_plugins(cuicui_layout_bevy_ui::Plugin);
        app.add_plugins(cuicui_chirp::loader::Plugin::new::<ImperiumDsl>());
        app.add_plugins(DefaultPickingPlugins);
        app.add_plugins(MirrorPlugin::<OnClick, UiAction>::new_from());
        app.register_type::<dsl::Mark>();
        app.register_type::<dsl::Group>();
        // custom ui modules
        app.add_plugins(HighlightPlugin);
        // Needed for text boxes and dynamic labels
        app.add_plugins(CuiCuiEguiPlugin);
        app.add_plugins(SwitchPlugin);

        app.add_systems(Startup, setup);
        //app.add_systems(Startup, ui_test_scene.after(setup));
    }
}

fn setup(mut cmds: Commands) {
    // Use LayoutRootCamera to mark a camera as the screen boundaries.
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 0.3;
    camera_bundle.transform.translation.x = 0.0;
    camera_bundle.transform.translation.y = 0.0;
    camera_bundle.transform.translation.z = 10.0;
    cmds.spawn((camera_bundle, LayoutRootCamera));
}

#[allow(dead_code)]
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
