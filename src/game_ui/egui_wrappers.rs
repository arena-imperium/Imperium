use bevy::ecs::reflect::ReflectComponent;
use bevy::prelude::{
    App, Component, Deref, DerefMut, GlobalTransform, Plugin, Query, Reflect, ResMut, Resource,
    Update, ViewVisibility,
};
use bevy::ui::Node;
use bevy::utils::HashMap;
use bevy_egui::egui::Align2;
use bevy_egui::{egui, EguiContexts};

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct EguiTextBox {
    pub(crate) id: String,
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct EguiLabel {
    pub(crate) id: String,
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
    query: Query<(&EguiTextBox, &Node, &GlobalTransform, &ViewVisibility)>,
    mut contexts: EguiContexts,
    mut text_map: ResMut<StrMap>,
) {
    let egui_context = contexts.ctx_mut();
    for (tex_box, ui_node, trnsfrm, visibility) in &query {
        if !visibility.get() {
            continue;
        };
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

fn draw_label(
    mut query: Query<(&EguiLabel, &Node, &GlobalTransform, &ViewVisibility)>,
    mut contexts: EguiContexts,
    mut text_map: ResMut<StrMap>,
) {
    let egui_context = contexts.ctx_mut();
    for (tex_box, ui_node, trnsfrm, visibility) in &mut query {
        if !visibility.get() {
            continue;
        };
        let node_pos = ui_node.logical_rect(trnsfrm).center();

        egui::Area::new(tex_box.id.clone())
            .fixed_pos(egui::Pos2::new(node_pos.x, node_pos.y))
            .pivot(Align2::CENTER_CENTER)
            .show(egui_context, |ui| {
                let text_ref = text_map.entry(tex_box.id.clone()).or_default();
                ui.label(text_ref.clone());
            });
    }
}
