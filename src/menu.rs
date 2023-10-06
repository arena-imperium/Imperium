use bevy::prelude::Commands;
use bevy_egui::{egui, EguiContext};
use bevy_egui::egui::Ui;
use crate::{Scene};
use crate::solana::HologramServer;

pub fn dev_menu(ui: &mut Ui, server: &mut HologramServer, commands: &mut Commands) {
    if ui.button("Init Realm").clicked() {
        server
            .fire_default_initialize_realm_task(commands);
    }
    if ui.button("Create User Account").clicked() {
        server
            .fire_default_create_user_account_task(commands);
    }
    if ui.button("Create Spaceship").clicked() {
        server
            .fire_default_create_spaceship_task(commands);
    }
    if ui.button("Join Arena Matchmaking Queue").clicked() {
        //server.fire_arena_matchmaking_task(commands);
    }
}

pub fn login_window(egui: &mut EguiContext, server: &mut HologramServer) -> Option<Scene>{
    /*let window_width = 200.0;
    let window_height = 100.0;

    let screen_rect = egui.screen_rect();
    let window_start_x = (screen_rect.width() - window_width) / 2.0;
    let window_start_y = (screen_rect.height() - window_height) / 2.0;

    let window_rect = egui::Rect::from_min_size(
        egui::pos2(window_start_x, window_start_y),
        egui::vec2(window_width, window_height)
    );
    let mut new_scene = None;
    egui::Window::new("Centered Window")
        .fixed_pos(window_rect.min)
        .fixed_size(window_rect.size())
        .title_bar(false)
        .resizable(false)
        .show(c.egui, |ui| {

                ui.label("Hanger entry id request");

            ui.horizontal_centered(|ui| {
                if ui.button("Cancel").clicked(){
                    new_scene = Scene::Login(Login::NotLoggedIn)
                }
                // Add any additional UI elements here
                if ui.button("Connect Pilot ID").clicked(){
                    new_scene = Scene::Hanger
                }
            });
        });
    new_scene*/
    None
}
