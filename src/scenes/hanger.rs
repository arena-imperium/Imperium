use crate::game_ui::dsl::{OnClick, UiAction};
use crate::solana::HologramServer;
use crate::Scene;
use bevy::log;
use bevy::prelude::*;
use cuicui_chirp::ChirpBundle;
use cuicui_dsl::dsl;
use cuicui_layout::dsl_functions::{child, pct};
use cuicui_layout_bevy_ui::UiDsl;
use hologram::state::SpaceShip;

pub struct HangerScenePlugin;

impl Plugin for HangerScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, hanger.run_if(in_state(Scene::Hanger)));
        app.add_systems(OnEnter(Scene::Hanger), on_hanger_init);
        app.add_systems(OnExit(Scene::Hanger), on_hanger_exit);
    }
}
fn hanger(mut init: Local<bool>) {
    if !*init {
        log::info!("Inside hanger");
        *init = true;
    }
}

#[derive(Default, Component)]
pub struct HangerUi;

#[derive(Default, Component)]
pub struct HangerSceneObj;

// Setup the scene for when the station is focused on
pub fn on_hanger_init(
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
    server: Option<Res<HologramServer>>, // mut text_map: ResMut<StrMap>,
    mut next_state: ResMut<NextState<Scene>>,
) {
    'outer: {
        if let Some(server) = server {
            if let Some(account) = &server.user_account {
                for ship in &account.spaceships {
                    server.fire_fetch_account_task::<SpaceShip>(&mut cmds, &ship.spaceship);
                }
                // if both server and server.user_account are cached
                // we can start the process to load ship data.
                break 'outer;
            }
        }
        // if not, something went wrong with sign in; we should never
        // get here, so go back to the station scene.
        next_state.set(Scene::Station);
    }

    UiAction::add_action("buy_spaceship", || {
        // if this gets too big, split out into its own function
        OnClick::run(
            |mut next_state: ResMut<NextState<crate::Scene>>,
             server: Option<Res<HologramServer>>| {
                if let Some(server) = server {
                    // Show popup dialog asking for ship name

                    // server.fire_create_spaceship_task()
                } else {
                    // if_no_server();
                }
            },
        )
    });

    log::info!("hanger init");
    cmds.spawn((
        ChirpBundle::new(asset_server.load("ui/chirps/hanger_menu.chirp")),
        HangerSceneObj,
    ));
    cmds.spawn((
        ChirpBundle::new(asset_server.load("ui/chirps/hangar_popup.chirp")),
        HangerSceneObj,
    ));
}

// Despawn scene
pub fn on_hanger_exit(
    mut cmds: Commands,
    ui: Query<Entity, With<HangerUi>>,
    hanger_scene: Query<Entity, With<HangerSceneObj>>,
) {
    cmds.entity(ui.iter().next().unwrap()).despawn_recursive();
    for entity in &hanger_scene {
        cmds.entity(entity).despawn();
    }
}
