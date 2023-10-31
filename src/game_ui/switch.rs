use crate::game_ui::dsl::{Group, Mark};
use bevy::prelude::*;
use bevy::utils::HashMap;

/// A plugin for switching between active ui trees.
///
/// Put [`Mark`]() components on ui elements, and
/// invisible on all elements you don't want to be default.
///
/// [`Group`]()'d elements  will ensure only that group's visibility is affected
///
/// Send a [`SwitchToUI`] event with text indicating the MarkUI() you want active
///
pub struct SwitchPlugin;

impl Plugin for SwitchPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SwitchToUI>();
        app.add_systems(PostUpdate, track_added_marks);
        app.add_systems(PostUpdate, track_removed_marks);
        app.add_systems(PostUpdate, change_visibility_system);
    }
}

struct SwitchManager {
    marks: HashMap<String, Entity>,
}

pub struct SwitchToUI {
    pub target: String,
}
impl SwitchToUI {
    pub fn new(target_ui: String) -> Self {
        Self { target: target_ui }
    }
}

fn track_added_marks(
    mut mark_manager: ResMut<SwitchManager>,
    query: Query<(Entity, &Mark), Added<Mark>>,
) {
    for (entity, mark) in query.iter() {
        mark_manager.marks.insert(mark.0.clone(), entity);
    }
}

fn track_removed_marks(
    mut mark_manager: ResMut<SwitchManager>,
    query: Query<Entity, RemovedComponents<Mark>>,
) {
    for entity in query.iter() {
        mark_manager.marks.retain(|_, &v| v != entity);
    }
}

fn change_visibility_system(
    mut ev_change_visibility: EventReader<SwitchToUI>,
    mark_manager: Res<SwitchManager>,
    mut query: Query<(&mut Visibility, Option<&Group>), With<Mark>>,
) {
    for ev in ev_change_visibility.iter() {
        if let Some(target_entity) = mark_manager.marks.get(&ev.target_mark) {
            let mut target_group: Option<String> = None;

            // Find the group of the target entity
            if let Ok((_, group)) = query.get(*target_entity) {
                if let Some(group) = group {
                    target_group = Some(group.0.clone());
                }
            }

            // Update visibility
            for (mut visibility, group) in query.iter_mut() {
                if let Some(target_group) = &target_group {
                    // Make sure both entities belong to the same group
                    // and mark the entity invisible if it does.
                    if let Some(actual_group) = &group {
                        if let Some(target_group_str) = &target_group {
                            if actual_group.0 == *target_group_str {
                                *visibility = Visibility::Hidden;
                            }
                        }
                    }
                } else {
                    if group.is_none() {
                        *visibility = Visibility::Hidden;
                    }
                }
            }

            // Set the target entity's visibility to Inherited
            if let Ok(mut visibility) = query.get_mut(*target_entity) {
                *visibility = Visibility::Inherited;
            }
        }
    }
}
