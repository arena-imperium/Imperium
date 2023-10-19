use bevy::ecs::system::EntityCommands;
use bevy::log;
use bevy::prelude::{ReflectComponent, Color, Component, Entity, Font, Handle, NodeBundle, Reflect, Style, TextBundle, TextStyle, UiRect, Val, Commands, ResMut, NextState};
use cuicui_chirp::parse_dsl_impl;
use cuicui_dsl::DslBundle;
use cuicui_layout_bevy_ui::UiDsl;
use bevy_mod_picking;
use bevy_mod_picking::prelude::{Click, On, Pointer};
use crate::game_ui::highlight::Highlight;
use crate::game_ui::Scene;

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub enum UiAction {
    #[default]
    None,
    PrintHello,
    PrintGoodbye,
    TransitionScene(Scene)
}

pub type OnClick = On<Pointer<Click>>;

// Converts UiAction struct into a command/single run system,
impl<'a> From<&'a UiAction> for OnClick {
    fn from(value: &'a UiAction) -> Self {
        match value {
            UiAction::PrintHello => {
                OnClick::run(|cmds: Commands| log::info!("Hello world!"))
            },

            UiAction::PrintGoodbye => {
                OnClick::run(|cmds: Commands| log::info!("Farewell, odious world!"))
            },
            UiAction::None => {
                OnClick::run(||{log::info!("Nothing happened")})
            }
            UiAction::TransitionScene(scene) => {
                OnClick::run(|cmds: Commands, mut next_state: ResMut<NextState<Scene>>| {next_state.set(*scene);})
            }
        }
    }
}

pub struct ImperiumDsl {
    inner: UiDsl,
    // Need a variable here that encapsulates all the different kinds of actions
    is_button: bool,
    is_text_box: bool,
    is_highlightable: bool,
    is_label: bool,
    /// Data shared by actions and text box's
    ///
    /// actions need to know what action is being executed, and
    /// text box's need to know where their contents are stored.
    data: Option<Box<str>>,
}

impl Default for ImperiumDsl {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            is_button: false,
            is_text_box: false,
            is_highlightable: false,
            is_label: false,
            data: None,
        }
    }
}
#[parse_dsl_impl(delegate = inner)]
impl ImperiumDsl {

    fn button(&mut self, text: &str) {
        self.is_button = true;
        self.data = Some(text.into());
    }

    /// allows dynamic text from egui key value par
    fn text_box(&mut self, text: &str) {
        self.is_text_box = true;
        self.data = Some(text.into());
    }
    fn highlight(&mut self) {
        self.is_highlightable = true;
    }

    /// Like the text box, allows dyinamic text from egui key value par
    /// but this time uneditable
    fn label(&mut self, text: &str) {
        self.is_label = true;
        self.data = Some(text.into());
    }
}


impl DslBundle for ImperiumDsl {
    fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
        if self.is_button {
            /// Todo: change the action inserted based on data contents
            cmds.insert(UiAction::None);
        }
        if self.is_highlightable {
            cmds.insert(Highlight::new(Color::DARK_GREEN));
        }
        if self.is_text_box {
            // Todo: insert an egui text box here that modifies the data stored
            // in key value par associated with data in egui's internal memory.
            //let mut node_bundle = bevy_ui::NodeBundle::default();
            //node_bundle.transform.
        }
        self.inner.insert(cmds)
    }
}