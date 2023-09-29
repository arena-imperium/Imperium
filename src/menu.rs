use {
    crate::GameContext,
    comfy::{egui::Ui, *},
};

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
