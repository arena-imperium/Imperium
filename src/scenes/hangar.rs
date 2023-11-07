use crate::game_ui::dsl::{OnClick, UiAction};
use crate::game_ui::egui_wrappers::StrMap;
use crate::solana::HologramServer;
use crate::Scene;
use bevy::log;
use bevy::prelude::*;
use cuicui_chirp::ChirpBundle;
use cuicui_dsl::dsl;
use cuicui_layout::dsl_functions::{child, pct};
use cuicui_layout_bevy_ui::UiDsl;
use hologram::state::SpaceShip;

pub struct HangarScenePlugin;

impl Plugin for HangarScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, hangar.run_if(in_state(Scene::Hangar)));
        app.add_systems(OnEnter(Scene::Hangar), on_hangar_init);
        app.add_systems(OnExit(Scene::Hangar), on_hangar_exit);
    }
}
fn hangar(mut init: Local<bool>) {
    if !*init {
        log::info!("Inside hangar");
        *init = true;
    }
}

#[derive(Default, Component)]
pub struct HangarUi;

#[derive(Default, Component)]
pub struct HangarSceneObj;
#[derive(Default, Component)]
pub struct NewShipDialog;

// Setup the scene for when the station is focused on
pub fn on_hangar_init(
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
    server: Option<Res<HologramServer>>, // mut text_map: ResMut<StrMap>,
    mut next_state: ResMut<NextState<Scene>>,
    mut text_map: ResMut<StrMap>,
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
        OnClick::run(|mut cmds: Commands, asset_server: Res<AssetServer>| {
            cmds.spawn((
                ChirpBundle::new(asset_server.load("ui/chirps/hangar_popup.chirp")),
                HangarSceneObj,
                NewShipDialog,
            ));
        })
    });

    UiAction::add_action("cancel", || {
        OnClick::run(
            |mut cmds: Commands, popup: Query<Entity, With<NewShipDialog>>| {
                cmds.entity(popup.iter().next().unwrap())
                    .despawn_recursive();
            },
        )
    });

    text_map.insert("new_ship_name".to_owned(), "DefaultShip".to_owned());
    UiAction::add_action("confirm", || {
        OnClick::run(
            |mut cmds: Commands,
             popup: Query<Entity, With<NewShipDialog>>,
             server: Option<Res<HologramServer>>,
             mut text_map: ResMut<StrMap>| {
                server.as_ref().unwrap().fire_create_spaceship_task(
                    &mut cmds,
                    text_map.get("new_ship_name").unwrap(),
                    &server.as_ref().unwrap().calc_realm_pda().0,
                    &server.as_ref().unwrap().user_account.as_ref().unwrap().user,
                );
                cmds.entity(popup.iter().next().unwrap())
                    .despawn_recursive();
            },
        )
    });

    log::info!("hangar init");
    cmds.spawn((
        ChirpBundle::new(asset_server.load("ui/chirps/hangar_menu.chirp")),
        HangarSceneObj,
    ));
}

// Despawn scene
pub fn on_hangar_exit(
    mut cmds: Commands,
    ui: Query<Entity, With<HangarUi>>,
    hangar_scene: Query<Entity, With<HangarSceneObj>>,
) {
    cmds.entity(ui.iter().next().unwrap()).despawn_recursive();
    for entity in &hangar_scene {
        cmds.entity(entity).despawn();
    }
}
