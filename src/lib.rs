#![allow(clippy::type_complexity)]


// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(Default, Clone, Eq, PartialEq, Debug, Hash)]
enum Scene {
    // Starting scene, where the player can setup a connection with their wallet
    #[default]
    Login,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

pub struct GamePlugin;

// Todo: use crossbeam to handle parallel tasks?
/*fn solana_transaction_task_handler(
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
}*/
