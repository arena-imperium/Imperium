// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod menu;
mod solana;

use {
    crate::{
        menu::dev_menu,
        solana::{solana_transaction_task_handler, HologramServer},
    },
    bevy_tasks::{IoTaskPool, TaskPool, TaskPoolBuilder},
    comfy::*,
};

comfy_game!(
    "Imperium",
    GameContext,
    GameState,
    make_context,
    setup,
    update
);

/// Ie: what gamemode/scene are we currently in?
#[derive(Default, Clone, Eq, PartialEq, Debug, Hash)]
enum Scene {
    #[default]
    Loading,
    // Starting scene, where the player can setup a connection with their wallet
    Login,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

pub struct GameState {
    /* structures for tracking sengine state should go here*/
    // Note this is different from GameContext in that game context
    // is a convenient *reference*, whereas this is where the data is
    // actually located.
    tasks: TaskPool,
    scene: Scene,
    solana_server: HologramServer,
}

impl GameState {
    pub fn new(_c: &mut EngineContext) -> Self {
        Self {
            tasks: TaskPoolBuilder::new()
                .thread_name("MainThreadPool".to_string())
                .build(),
            scene: Default::default(),
            solana_server: Default::default(),
        }
    }
}

/// Used to reference engine internal state mutably
/// and act as convenient access to common internal state.
pub struct GameContext<'a, 'b: 'a> {
    // While we could access delta through .engine, it's easier to just expose it once and then
    // benefit all over the codebase.
    // We could just write c.engine.egui instead, but ... getting in the habit
    // of re-exporting things into the `GameContext` usually ends up being nice.
    pub egui: &'a egui::Context,
    pub engine: &'a mut EngineContext<'b>,
    pub tasks: &'a mut TaskPool,
    pub solana_server: &'a mut HologramServer,
}

fn make_context<'a, 'b: 'a>(
    state: &'a mut GameState,
    engine: &'a mut EngineContext<'b>,
) -> GameContext<'a, 'b> {
    GameContext {
        egui: engine.egui,
        engine,
        tasks: &mut state.tasks,
        solana_server: &mut state.solana_server,
    }
}

/// Setup initial state of the engine, load assets, etc.
fn setup(_c: &mut GameContext) {
    // Initializes the task pool
    IoTaskPool::init(|| TaskPoolBuilder::new().build());
}

/// Called every frame; our main loop.
///
/// Drawing and most things are immediate mode; so can be very
/// quick to setup ui for debugging state.
fn update(c: &mut GameContext) {
    draw_text(
        "Welcome to the Imperium galactic Arena!",
        Position::screen_percent(0.5, 0.85).to_world(),
        WHITE,
        TextAlign::Center,
    );

    egui::Window::new("Dev Test Window")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(c.egui, |ui| {
            dev_menu(ui, c);
        });

    // Handles solana threads and such for us.
    solana_transaction_task_handler(
        &mut c.engine.commands.borrow_mut(),
        &mut c.engine.world.borrow_mut(),
    );
}
