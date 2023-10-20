use bevy::utils::HashMap;
use bevy::prelude::{Reflect, App, Commands, Component, Entity, Plugin, Query, Update, GlobalTransform, Resource, Deref, DerefMut, ResMut, PreUpdate};
use bevy::ui::Node;
use bevy_egui::{egui, EguiContexts};
use bevy_egui::egui::{Align2, Frame};
use bevy::ecs::reflect::ReflectComponent;
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct EguiTextBox {
    pub(crate) id: String,
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct EguiLabel {
    pub(crate) egui_text_ref: String,
}
/// Maps a string reference (used to refer to data in hot reloaded chirp files)
/// to a mutable string (used as entry field or label for something that changes
/// at runtime)
#[derive(Resource, Deref, DerefMut, Default)]
pub struct StrMap(HashMap<String, String>);

pub struct CuiCuiEguiPlugin;

impl Plugin for CuiCuiEguiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StrMap::default());

        app.register_type::<EguiTextBox>();
        app.add_systems(Update, draw_text_box);
        app.register_type::<EguiLabel>();
        app.add_systems(Update, draw_label);
    }
}

fn draw_text_box(
    mut cmds: Commands,
    mut query: Query<(Entity, &EguiTextBox, &Node, &GlobalTransform)>,
    mut contexts: EguiContexts,
    mut text_map: ResMut<StrMap>,
) {
    let egui_context = contexts.ctx_mut();
    for (entity, tex_box, ui_node, trnsfrm) in &mut query {
        let node_pos = ui_node.logical_rect(trnsfrm).center();

        egui::Area::new(tex_box.id.clone())
            .fixed_pos(egui::Pos2::new(node_pos.x, node_pos.y))
            .pivot(Align2::CENTER_CENTER)
            .show(egui_context, |ui| {
                    let text_ref = text_map.entry(tex_box.id.clone()).or_default();
                    ui.text_edit_singleline(text_ref);
            });
    }
}

fn draw_label(mut cmds: Commands, query: Query<(Entity, &mut EguiLabel, &Node)>, mut contexts: EguiContexts) {
    for entity in &query {

    }
}