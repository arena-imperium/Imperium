use {
    crate::{loading::FontAssets, solana::HologramServer, GameState},
    bevy::prelude::*,
    std::borrow::BorrowMut,
};

#[derive(Component)]
pub struct MenuRoot;

#[derive(Component, Clone, Copy, Debug)]
pub enum MenuButton {
    Play,
    InitializeRealm,
    CreateUserAccount,
    CreateSpaceship,
}

pub struct MenuPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonColors>()
            .add_systems(OnEnter(GameState::Menu), create_menu)
            .add_systems(Update, button_clicked.run_if(in_state(GameState::Menu)))
            .add_systems(OnExit(GameState::Menu), cleanup_menu);
    }
}

#[derive(Resource)]
struct ButtonColors {
    normal: Color,
    hovered: Color,
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            normal: Color::rgb(0.15, 0.15, 0.15),
            hovered: Color::rgb(0.25, 0.25, 0.25),
        }
    }
}

fn create_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    button_colors: Res<ButtonColors>,
) {
    let buttons = [
        (MenuButton::Play, "Play"),
        (MenuButton::InitializeRealm, "Init Realm"),
        (MenuButton::CreateUserAccount, "Create User Account"),
        (MenuButton::CreateSpaceship, "Create Spaceship"),
    ];
    let button_text_style = TextStyle {
        font: font_assets.fira_sans.clone(),
        font_size: 40.0,
        color: Color::rgb(0.9, 0.9, 0.9),
    };
    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                flex_basis: Val::Percent(30.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceEvenly,
                ..Default::default()
            },
            background_color: BackgroundColor(Color::BLACK),
            ..Default::default()
        })
        .insert(MenuRoot)
        .with_children(|parent| {
            for button in buttons {
                parent
                    .spawn(ButtonBundle {
                        style: Style {
                            width: Val::Percent(80.),
                            justify_content: JustifyContent::SpaceEvenly,
                            flex_basis: Val::Percent(0.),
                            margin: UiRect::all(Val::Auto),
                            ..Default::default()
                        },
                        background_color: button_colors.normal.into(),
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            button.1,
                            button_text_style.clone(),
                        ));
                    })
                    .insert(button.0);
            }
        });
}

fn button_clicked(
    mut commands: Commands,
    hologam_server: Res<HologramServer>,
    button_colors: Res<ButtonColors>,
    mut state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, menu_button, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => match menu_button {
                MenuButton::Play => {
                    state.set(GameState::Playing);
                }
                MenuButton::InitializeRealm => {
                    hologam_server.default_initialize_realm(commands.borrow_mut());
                }
                MenuButton::CreateUserAccount => {
                    hologam_server.default_create_user_account(commands.borrow_mut());
                }
                MenuButton::CreateSpaceship => {
                    hologam_server.default_create_spaceship(commands.borrow_mut());
                }
            },
            Interaction::Hovered => {
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                *color = button_colors.normal.into();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu_root: Query<Entity, With<MenuRoot>>) {
    commands.entity(menu_root.single()).despawn_recursive();
}
