// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod menu;
mod solana;

use bevy_tasks::{TaskPool, TaskPoolBuilder};
use comfy::*;
use crate::menu::dev_menu;

// Todo:
// Delete almost everything else.
// consolidate into one .rs file
// Spin out new .rs files as needed to keep code easy to read.

// This example shows an integration between comfy and blobs, a simple 2d physics engine. It's not
// the most beautiful example, and maybe a bit verbose for what it does, but it tries to showcase
// some more extensible ways of using comfy.
comfy_game!(
    "Imperium",
    GameContext,
    GameState,
    make_context,
    setup,
    update
);

pub struct GameState {
    /* structures for tracking sengine state should go here*/
    // Note this is different from GameContext in that game context
    // is a conveniant *reference*, whereas this is where the data is
    // actually located.

    tasks: TaskPool,
}

impl GameState {
    pub fn new(c: &mut EngineContext) -> Self {
        Self { tasks: TaskPoolBuilder::new()
            .thread_name("MainThreadPool".to_string())
            .build()
        }
    }
}

/// referenced by doing c.egui, etc.
pub struct GameContext<'a, 'b: 'a> {
    // While we could access delta through .engine, it's easier to just expose it once and then
    // benefit all over the codebase.
    // We could just write c.engine.egui instead, but ... getting in the habit
    // of re-exporting things into the `GameContext` usually ends up being nice.
    pub egui: &'a egui::Context,
    pub engine: &'a mut EngineContext<'b>,
    pub tasks: &'a mut TaskPool,
}

fn make_context<'a, 'b: 'a>(
    state: &'a mut GameState,
    engine: &'a mut EngineContext<'b>,
) -> GameContext<'a, 'b> {
    GameContext {
        egui: engine.egui,
        engine,
        tasks: &mut state.tasks,
    }
}

// Setup initial state of the engine, load assets, etc.
fn setup(c: &mut GameContext) {
    // We'll need SFX for this
    c.engine.load_sound_from_bytes(
        // Every sound gets a string name later used to reference it.
        "comfy-flying",
        include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/audio/flying.ogg"
        )),
        StaticSoundSettings::default(),
    );
}

fn update(c: &mut GameContext) {

    /*draw_circle(
        collider.absolute_transform.translation,
        collider.radius,
        BLUE.alpha(0.5),
        4,
    );*/

    draw_text(
        "This is test ingame text.",
        Position::screen_percent(0.5, 0.85).to_world(),
        WHITE,
        TextAlign::Center,
    );

    egui::Window::new("Test Window")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(c.egui, |ui| {
            dev_menu(ui, c);
        });
}


/*fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Imperium".to_string(),
                resolution: (1024., 780.).into(),
                // Bind to canvas included in `index.html`
                canvas: Some("#bevy".to_owned()),
                // Tells wasm not to override default event handling, like F5 and Ctrl+R
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(GamePlugin)
        .add_systems(Startup, set_window_icon)
        .run();
}*/

// Sets the icon on windows and X11
/*fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let primary_entity = primary_window.single();
    let primary = windows.get_window(primary_entity).unwrap();
    let icon_buf = Cursor::new(include_bytes!(
        "../build/macos/AppIcon.iconset/icon_256x256.png"
    ));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}*/
