use {
    crate::solana::HologramServer,
    bevy::{
        log,
        prelude::{App, Commands, Plugin, Res, Update},
    },
    bevy_egui::{egui, EguiContexts, EguiPlugin},
};

pub struct DevUI;
impl Plugin for DevUI {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }
        app.add_systems(Update, dev_ui);
    }
}

pub fn dev_ui(mut contexts: EguiContexts, server: Res<HologramServer>, mut commands: Commands) {
    let egui_context = contexts.ctx_mut();
    egui::Window::new("Dev Test Window")
        .default_pos(egui::Pos2::new(0.0, 0.0))
        .show(egui_context, |ui| {
            if ui.button("Init Realm").clicked() {
                server.fire_default_initialize_realm_task(&mut commands);
            }
            if ui.button("Create User Account").clicked() {
                server.fire_default_create_user_account_task(&mut commands);
            }
            if ui.button("Create Spaceship").clicked() {
                server.fire_default_create_spaceship_task(&mut commands);
            }
            if ui.button("Pick Crate").clicked() {
                server.fire_default_pick_crate_task(&mut commands);
            }
            if ui.button("Join Arena Matchmaking Queue").clicked() {
                server.fire_default_arena_matchmaking_task(&mut commands);
            }
            if ui.button("Reset").clicked() {
                //*c.scene = Scene::Loading
                log::info!("button click test was successful, yay!")
            }
        });
}
