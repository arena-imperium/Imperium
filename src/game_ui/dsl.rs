use std::sync::RwLock;

use bevy::ecs::system::EntityCommands;
use bevy::log;
use bevy::prelude::{Color, Component, Entity, Reflect, ReflectComponent, Visibility};
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
    #[derive(Debug)]
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
}

pub type OnClick = On<Pointer<Click>>;

// Converts UiAction struct into a command/single run system
impl<'a> From<&'a UiAction> for OnClick {
    fn from(value: &'a UiAction) -> Self {
        let map = ON_CLICK_MAP.read().unwrap();
        if let Some(func) = map.get(value.action_lookup.as_str()) {
            func()
        } else {
            log::warn!(
                "Action map assignment failed!, action name: {}, map contents: {:?}",
                value.action_lookup,
                ON_CLICK_MAP,
            );
            OnClick::run(|| log::info!("Nothing happened"))
        }
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Mark(pub String);
/*fn parse_mark<T>(reg: &TypeRegistry, _: T, input: &str) -> Result<Mark, anyhow::Error> {
    Ok(Mark(input.to_owned()))
}*/

pub struct ImperiumDsl {
    inner: UiDsl,
    // Need a variable here that encapsulates all the different kinds of actions
    is_button: bool,
    is_text_box: bool,
    is_highlightable: bool,
    is_label: bool,
    is_hidden: bool,
    mark: Option<Box<str>>,
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
            is_hidden: false,
            mark: None,
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

    fn highlight(&mut self) {
        self.is_highlightable = true;
    }

    /// Sets this ui element to be hidden.
    /// (as a result, all its child elemnts will also be hidden)
    fn hidden(&mut self) {
        self.is_hidden = true;
    }

    /// Attaches a Mark(String) component to this entity.
    /// Useful when you want to do something to a specific ui entity.
    fn mark(&mut self, mark: &str) {
        self.mark = Some(mark.into())
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
        if let Some(data) = self.mark.take() {
            cmds.insert(Mark(data.into()));
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
        let id = self.inner.insert(cmds);
        // By adding this *after* insert(cmds) on inner, we ensure
        // Visibility is added after nodebundle (which adds its own visibility)
        // is added by the UiDsl, so that we overwrite it.
        //
        if self.is_hidden {
            cmds.insert(Visibility::Hidden);
        }
        id
    }
}
