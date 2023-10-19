use bevy::ecs::system::EntityCommands;
use bevy::log;
use bevy::prelude::{ReflectComponent, BuildChildren, Color, Component, Entity, Font, Handle, NodeBundle, Reflect, Style, TextBundle, TextStyle, UiRect, Val, Commands};
use cuicui_chirp::parse_dsl_impl;
use cuicui_dsl::DslBundle;
use cuicui_layout_bevy_ui::UiDsl;
use bevy_mod_picking;
use bevy_mod_picking::prelude::{Click, On, Pointer};

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub enum UiAction {
    #[default]
    None,
    PrintHello,
    PrintGoodbye,
}

pub type OnClick = On<Pointer<Click>>;

impl<'a> From<&'a UiAction> for OnClick {
    fn from(value: &'a UiAction) -> Self {
        match value {
            /*UiAction::LogInfo(text) => {
                let text = text.clone();
                OnClick::run(move || info!("{text}"))
            }
            &UiAction::EmitSwitchTab(index) => {
                OnClick::run(move |mut ev: EventWriter<_>| ev.send(SwitchTab(index)))
            }
            &UiAction::EmitSwitchGraph(index) => {
                OnClick::run(move |mut ev: EventWriter<_>| ev.send(SwitchGraph(index)))
            }
            ReflectOnClick::Invalid => unreachable!("Should never spawn an invalid ReflectOnClick"),*/

            UiAction::PrintHello => {
                OnClick::run(|cmds: Commands| log::info!("Hello world!"))
            },

            UiAction::PrintGoodbye => {
                OnClick::run(|cmds: Commands| log::info!("Farewell, odious world!"))
            },
            UiAction::None => {
                OnClick::run(||{log::info!("Nothing happened")})
            }
        }
    }
}

pub struct ImperiumDsl {
    inner: UiDsl,
    is_action: bool,
}

impl Default for ImperiumDsl {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            is_action: false,
        }
    }
}
#[parse_dsl_impl(delegate = inner)]
impl ImperiumDsl {
    fn actionable(&mut self) {
        self.is_action = true;
    }
}


impl DslBundle for ImperiumDsl {
    fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
        if self.is_action {
            cmds.insert(UiAction::None);
        }
        self.inner.insert(cmds)
    }
}