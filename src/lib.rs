#![allow(clippy::type_complexity)]

mod actions;
mod audio;
mod loading;
mod menu;
mod player;
mod solana;

#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use {
    crate::{
        actions::ActionsPlugin, audio::InternalAudioPlugin, loading::LoadingPlugin,
        menu::MenuPlugin, player::PlayerPlugin,
    },
    bevy::{app::App, log, prelude::*},
    futures_lite::future,
    solana::{HologramServer, SolanaTransactionTask},
};

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HologramServer>()
            .add_state::<GameState>()
            .add_plugins((
                LoadingPlugin,
                MenuPlugin,
                ActionsPlugin,
                InternalAudioPlugin,
                PlayerPlugin,
            ))
            .add_systems(Update, solana_transaction_task_handler);

        #[cfg(debug_assertions)]
        {
            app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }
}

fn solana_transaction_task_handler(
    mut commands: Commands,
    mut solana_transaction_tasks: Query<(Entity, &mut SolanaTransactionTask)>,
) {
    for (entity, mut task) in &mut solana_transaction_tasks {
        match future::block_on(future::poll_once(&mut task.task)) {
            Some(result) => {
                let status = match result {
                    Ok(confirmed_transaction) => {
                        match confirmed_transaction.transaction.meta.unwrap().err {
                            Some(error) => {
                                format!("Transaction failed: {}", error)
                            }
                            None => "Transaction succeeded".to_string(),
                        }
                    }
                    Err(error) => {
                        format!("Transaction failed: {}", error)
                    }
                };
                let message = format!("{}: {}", task.description, status);
                log::info!("{}", message);
                commands.entity(entity).despawn();
            }
            None => {}
        };
    }
}
