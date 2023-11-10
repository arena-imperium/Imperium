use crate::solana::generate_test_client;
use bevy::prelude::ResMut;
use hologram::state::{Hull, SpaceShipLite};
use hologram::utils::LimitedString;
pub use solana_program::pubkey::Pubkey;
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

pub fn dev_ui(
    mut contexts: EguiContexts,
    mut server: Option<ResMut<HologramServer>>,
    mut commands: Commands,
) {
    let egui_context = contexts.ctx_mut();
    egui::Window::new("Dev Test Window")
        .default_pos(egui::Pos2::new(0.0, 0.0))
        .show(egui_context, |ui| {
            if let Some(mut server) = server {
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
                if ui.button("Insert Test Spaceship").clicked() {
                    server
                        .user_account
                        .as_mut()
                        .unwrap()
                        .spaceships
                        .push(SpaceShipLite {
                            name: LimitedString::new("TestShipName"),
                            hull: Hull::RareOne,
                            spaceship: Pubkey::default(),
                        })
                }
                if ui.button("Reset").clicked() {
                    //*c.scene = Scene::Loading
                    log::info!("button click test was successful, yay!")
                }
            } else {
                if ui.button("Init HologramServer with test Client").clicked() {
                    commands.insert_resource(HologramServer::new(generate_test_client()));
                }
            }
        });
}
