// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod menu;
mod solana;
mod login;

use {
    crate::{
        menu::dev_menu,
        solana::{solana_transaction_task_handler, HologramServer},
    },
    bevy_tasks::{IoTaskPool, TaskPool, TaskPoolBuilder},
    comfy::*,
};
use crate::menu::login_window;

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
pub enum Scene {
    #[default]
    Loading,
    // Starting scene, where the player can setup a connection with their wallet
    Login(Login),
    // Here the menu is drawn and waiting for player interaction
    MainMenu,
}

#[derive(Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Login{
    #[default]
    NotLoggedIn,
    LoginWindow,
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
    pub scene: &'a mut Scene
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
        scene: &mut state.scene,
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
    egui::Window::new("Dev Test Window")
        .default_pos(egui::Pos2::new(0.0, 0.0))
        .show(c.egui, |ui| {
            dev_menu(ui, c);
            if ui.button("Reset").clicked(){
                *c.scene = Scene::Loading
            }
        });
    clear_background(BLACK);
    match c.scene {
        Scene::Loading => {
            draw_text(
                "Loading...",
                Position::screen_percent(0.5, 0.5).to_world(),
                WHITE,
                TextAlign::Center,
            );
            *c.scene = Scene::Login(Login::NotLoggedIn)
        }
        Scene::Login(login_state) => {
            clear_background(BLACK);
            let size = 2.0;

            draw_circle(vec2(0.0, 0.0), size, Color::new(0.45, 0.8, 0.11, 1.00), 2);
            draw_quad(vec2(0.0, 0.0), vec2(1.75, 0.25), get_time() as f32, Color::new(0.41, 0.46, 0.47, 1.00), 3, texture_id("1px"), Vec2::ZERO);


            match login_state {
                Login::NotLoggedIn => {
                    if screen_to_world(mouse_screen()).distance(vec2(0.0, 0.0)) < size{
                        draw_circle_outline(vec2(0.0, 0.0), size, 0.1, ORANGE, 0);
                        if is_mouse_button_pressed(MouseButton::Left){
                            *c.scene = Scene::Login(Login::LoginWindow)
                        }
                    }
                }
                Login::LoginWindow => {
                    let window_scale = Position::screen_percent(0.25, 0.5).to_world();
                    // Todo: add a ui_utils struct to context, put useful variables there like padding
                    // and functions for making buttons etc.
                    let padding = egui_scale_factor()*0.35;
                    // Todo: replace with ui panel art
                    /*draw_sprite(texture_id("1px"), vec2(0.0, 0.0), Color::new(0.31, 0.31, 0.31, 1.00), 5, window_scale);*/
                    /*draw_text_ex(
                        &format!("Hanger Entry Id Request"),
                        window_scale*0.5 - Vec2::new(0.0, padding),
                        TextAlign::TopRight,
                        TextParams::default()
                    );*/
                    login_window(c);
                }
            }
        }
        Scene::MainMenu => {
            let top_pos = Position::screen_percent(0.95, 0.05).to_world();
            draw_text(
                &format!("Connected Wallet: {}", c.solana_server.admin_pubkey),
                top_pos,
                GREEN,
                TextAlign::TopRight,
            );
            draw_text(
                &format!("Account: {:05}", (random()*10000.0) as u32),
                top_pos - Vec2::new(0.0, egui_scale_factor()*0.35),
                RED,
                TextAlign::TopRight,
            );
            draw_text(
                "Welcome to the Imperium galactic Arena!",
                Position::screen_percent(0.5, 0.85).to_world(),
                WHITE,
                TextAlign::Center,
            );
        }
    }


    // Handles solana threads and such for us.
    solana_transaction_task_handler(
        &mut c.engine.commands.borrow_mut(),
        &mut c.engine.world.borrow_mut(),
    );
}
