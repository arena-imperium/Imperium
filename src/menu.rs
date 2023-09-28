use {
    crate::GameContext,
    comfy::{egui::Ui, *},
};

pub struct MenuRoot;

#[derive(Clone, Copy, Debug)]
pub enum MenuButton {
    Play,
    InitializeRealm,
    CreateUserAccount,
    CreateSpaceship,
    JoinArenaMatchmakingQueue,
}

pub struct MenuPlugin;

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
    /*match MenuButton::Play {
        MenuButton::Play => {
            c.state.set(GameState::Playing);
        }
        MenuButton::InitializeRealm => {
            hologam_server.fire_default_initialize_realm_task(commands.borrow_mut());
        }
        MenuButton::CreateUserAccount => {
            hologam_server.fire_default_create_user_account_task(commands.borrow_mut());
        }
        MenuButton::CreateSpaceship => {
            hologam_server.fire_default_create_spaceship_task(commands.borrow_mut());
        }
        MenuButton::JoinArenaMatchmakingQueue => {
            // hologam_server.fire_arena_matchmaking_task(commands.borrow_mut());
        }
    }*/
}
