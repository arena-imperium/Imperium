use comfy::*;
use comfy::egui::Ui;
use crate::GameContext;

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

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
pub fn dev_menu(ui: &mut Ui, c: &mut GameContext){
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
