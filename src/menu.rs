use {
    crate::GameContext,
    comfy::{egui::Ui, *},
};
use crate::{Login, Scene};

pub fn dev_menu(ui: &mut Ui, c: &mut GameContext) {
    if ui.button("Init Realm").clicked() {
        c.solana_server
            .fire_default_initialize_realm_task(&mut c.engine.commands.borrow_mut());
    }
    if ui.button("Create User Account").clicked() {
        c.solana_server
            .fire_default_create_user_account_task(&mut c.engine.commands.borrow_mut());
    }
    if ui.button("Create Spaceship").clicked() {
        c.solana_server
            .fire_default_create_spaceship_task(&mut c.engine.commands.borrow_mut());
    }
    if ui.button("Join Arena Matchmaking Queue").clicked() {
        //c.solana_server.fire_arena_matchmaking_task(&mut c.engine.commands.borrow_mut());
    }
}

pub fn login_window(c: &mut GameContext){
    let window_width = 200.0;
    let window_height = 100.0;

    let screen_rect = c.egui.screen_rect();
    let window_start_x = (screen_rect.width() - window_width) / 2.0;
    let window_start_y = (screen_rect.height() - window_height) / 2.0;

    let window_rect = egui::Rect::from_min_size(
        egui::pos2(window_start_x, window_start_y),
        egui::vec2(window_width, window_height)
    );

    egui::Window::new("Centered Window")
        .fixed_pos(window_rect.min)
        .fixed_size(window_rect.size())
        .title_bar(false)
        .resizable(false)
        .show(c.egui, |ui| {

                ui.label("Hanger entry id request");

            ui.horizontal_centered(|ui| {
                if ui.button("Cancel").clicked(){
                    *c.scene = Scene::Login(Login::NotLoggedIn)
                }
                // Add any additional UI elements here
                if ui.button("Connect Pilot ID").clicked(){
                    *c.scene = Scene::MainMenu
                }
            });
        });
}
