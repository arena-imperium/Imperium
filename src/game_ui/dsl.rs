use std::sync::RwLock;

use bevy::ecs::system::EntityCommands;
use bevy::log;
use bevy::prelude::{Color, Component, Entity, Reflect, ReflectComponent};
use bevy::utils::HashMap;
use bevy_mod_picking;
use bevy_mod_picking::prelude::{Click, On, Pointer};
use cuicui_chirp::parse_dsl_impl;
use cuicui_dsl::DslBundle;
use cuicui_layout_bevy_ui::UiDsl;
use lazy_static::lazy_static;

use crate::game_ui::egui_wrappers::{EguiLabel, EguiTextBox};
use crate::game_ui::highlight::Highlight;

type OnClickFunction = Box<dyn Fn() -> OnClick + Send + Sync>;

// Lazy static is used here because there is no easy way to to keep track
// of the functions that are activated by a button press dynamically.
// If you can think of a better method let let me know
lazy_static! {
    static ref ON_CLICK_MAP: RwLock<HashMap<&'static str, OnClickFunction>> = {
        let m: HashMap<&'static str, OnClickFunction> = HashMap::new();
        RwLock::new(m)
    };
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct UiAction {
    action_lookup: String,
}

impl UiAction {
    pub fn new(action: String) -> Self {
        Self {
            action_lookup: action,
        }
    }

    /// usage:
    /// ```
    /// UiAction::add_action("NewAction", || OnClick::run(|| log::info!("This is a new action")));
    /// ```
    pub fn add_action<F: Fn() -> OnClick + 'static + Send + Sync>(
        action_name: &'static str,
        func: F,
    ) {
        let mut map = ON_CLICK_MAP.write().unwrap();
        map.insert(action_name, Box::new(func));
    }
    pub fn remove_action(action_name: &'static str) {
        let mut map = ON_CLICK_MAP.write().unwrap();
        map.remove(action_name);
    }
}

pub type OnClick = On<Pointer<Click>>;

// Converts UiAction struct into a command/single run system
impl<'a> From<&'a UiAction> for OnClick {
    fn from(value: &'a UiAction) -> Self {
        let map = ON_CLICK_MAP.read().unwrap();
        if let Some(func) = map.get(value.action_lookup.as_str()) {
            func()
        } else {
            OnClick::run(|| log::info!("Nothing happened"))
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
    ///
    /// Create with `text_box(field_id_str)`
    ///
    /// To read or modify the field, use the StrMap resource like:
    /// ```
    /// fn draw_label(
    ///     mut text_map: ResMut<StrMap>,
    /// ) {
    ///     let text_ref = text_map.entry(field_id_str).or_default();
    /// }
    /// ```
    fn text_box(&mut self, text: &str) {
        self.is_text_box = true;
        self.data = Some(text.into());
    }
    fn highlight(&mut self) {
        self.is_highlightable = true;
    }

    /// Like the text box, allows dyinamic text from egui key value par
    /// but this time uneditable
    ///
    /// Create with `text_box(field_id_str)`
    ///
    /// To read or modify the field, use the StrMap resource like:
    /// ```
    /// fn draw_label(
    ///     mut text_map: ResMut<StrMap>,
    /// ) {
    ///     let text_ref = text_map.entry(field_id_str).or_default();
    /// }
    /// ```
    fn label(&mut self, text: &str) {
        self.is_label = true;
        self.data = Some(text.into());
    }
}

impl DslBundle for ImperiumDsl {
    fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
        if self.is_button {
            if let Some(data) = self.data.take() {
                cmds.insert(UiAction::new(data.into()));
            }
        }
        if self.is_highlightable {
            cmds.insert(Highlight::new(Color::DARK_GREEN));
        }
        if self.is_text_box {
            if let Some(data) = self.data.take() {
                cmds.insert(EguiTextBox { id: data.into() });
            }
        }
        if self.is_label {
            if let Some(data) = self.data.take() {
                cmds.insert(EguiLabel { id: data.into() });
            }
        }
        self.inner.insert(cmds)
    }
}