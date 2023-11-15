use crate::game_ui::dsl::{Mark, UiAction};
use crate::solana::HologramServer;
use crate::Scene;
use bevy::log;
use bevy::prelude::{
    in_state, App, AssetServer, Commands, Component, DespawnRecursiveExt, Entity,
    IntoSystemConfigs, NextState, OnEnter, OnExit, Plugin, Query, Res, ResMut, Update, Visibility,
    With,
};
use cuicui_chirp::ChirpBundle;
use hologram::state::SpaceShip;

pub struct ReplayScenePlugin;
impl Plugin for ReplayScenePlugin {
    fn build(&self, app: &mut App) {
        /*app.add_systems(
            Update,
            (handle_create_spaceships, refresh_ship_info)
                .run_if(in_state(Scene::Replay))
                .before(hangar_loop),
        );*/
        app.add_systems(Update, replay_loop.run_if(in_state(Scene::Replay)));
        app.add_systems(OnEnter(Scene::Replay), on_replay_init);
        app.add_systems(OnExit(Scene::Replay), on_replay_exit);
    }
}

#[derive(Default, Component)]
pub struct ReplayUi;

#[derive(Default, Component)]
pub struct ReplaySceneObj;

pub fn on_replay_init(
    mut cmds: Commands,
    asset_server: Res<AssetServer>,
    server: Option<Res<HologramServer>>, // mut text_map: ResMut<StrMap>,
    mut next_state: ResMut<NextState<Scene>>,
) {
    'outer: {
        if let Some(server) = server {
            if let Some(account) = &server.user_account {
                for ship in &account.spaceships {
                    // Fetch ship dataaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
                    // Todo: cache this data that's returned from the server.
                    server.fire_fetch_account_task::<SpaceShip>(&mut cmds, &ship.spaceship);
                }
                // if both server and server.user_account are cached
                // we can start the process to load ship data.
                break 'outer;
            }
        }
        // if not, something went wrong with sign in; we should never
        // get here, so go back to the station scene.
        //next_state.set(Scene::Station);
    }

    log::info!("replay scene init");
    cmds.spawn((
        ChirpBundle::new(asset_server.load("ui/chirps/replay_ui.chirp")),
        ReplayUi,
    ));
}

pub fn replay_loop(
    mut cmds: Commands,
    mut query: Query<(&mut Visibility, &Mark)>,
    server: Res<HologramServer>,
) {
}

pub fn on_replay_exit(
    mut cmds: Commands,
    ui: Query<Entity, With<ReplayUi>>,
    hangar_scene: Query<Entity, With<ReplaySceneObj>>,
) {
    cmds.entity(ui.iter().next().unwrap()).despawn_recursive();
    for entity in &hangar_scene {
        cmds.entity(entity).despawn();
    }
    UiAction::clear_actions();
}
