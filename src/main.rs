// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod asset_loading;
mod dev_ui;
mod game_ui;
pub mod input_util;
mod scenes;
mod solana;

use crate::asset_loading::AssetLoadingPlugin;
use crate::dev_ui::DevUI;
use crate::scenes::ScenesPlugin;
use crate::solana::SolanaPlugin;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use game_ui::GameGuiPlugin;
use image::load;
use image::ImageFormat::Png;
use {
    bevy::{prelude::*, window::PrimaryWindow, winit::WinitWindows, DefaultPlugins},
    bevy_inspector_egui::quick::WorldInspectorPlugin,
    std::io::Cursor,
    winit::window::Icon,
};

fn main() {
    let mut app = App::new();
    app.insert_resource(Msaa::Off);
    app.insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)));
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
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
            })
            // prevents blurry sprites
            .set(ImagePlugin::default_nearest()),
    );

    app.add_systems(Startup, set_window_icon);
    app.add_plugins(WorldInspectorPlugin::new());
    app.add_plugins(AssetLoadingPlugin);
    app.add_plugins(SolanaPlugin);
    app.add_plugins(GameGuiPlugin);
    app.add_plugins(ScenesPlugin);
    app.add_plugins(DevUI);
    #[cfg(debug_assertions)]
    {
        app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
    }
    app.run();
}

// Sets the icon on windows and X11
fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let primary_entity = primary_window.single();
    let primary = windows.get_window(primary_entity).unwrap();
    let icon_buf = Cursor::new(include_bytes!(
        "../build/macos/AppIcon.iconset/icon_256x256.png"
    ));
    if let Ok(image) = load(icon_buf, Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}

/*
/// Called every frame; our main loop.
///
/// Drawing and most things are immediate mode; so can be very
/// quick to setup ui for debugging state.
fn update() {
    match c.scene {
        Scene::Loading => {

        }
        Scene::Login(login_state) => {

            match login_state {
                Login::NotLoggedIn => {

                }
                Login::LoginWindow => {
                        /*&format!("Hangar Entry Id Request"),*/
                    login_window(c);
                }
            }
        }
        Scene::MainMenu => {
                /* &format!("Connected Wallet: {}", c.solana_server.admin_pubkey),
                &format!("Account: {:05}", (random()*10000.0) as u32),
                "Welcome to the Imperium galactic Arena!" */
        }
    }
}
 */

/// Ie: what gamemode/scene are we currently in?
#[derive(Default, Clone, Eq, PartialEq, Debug, Hash, States, Copy)]
pub enum Scene {
    #[default]
    Loading,
    /// Starting scene, where the player can setup a connection with their wallet
    Station,
    /// Here the menu is drawn and waiting for player interaction
    Hangar,
}
