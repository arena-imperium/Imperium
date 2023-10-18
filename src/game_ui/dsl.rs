use bevy::ecs::system::EntityCommands;
use bevy::prelude::{BuildChildren, Color, Entity, Font, Handle, NodeBundle, Style, TextBundle, TextStyle, UiRect, Val};
use cuicui_chirp::parse_dsl_impl;
use cuicui_dsl::DslBundle;
use cuicui_layout_bevy_ui::UiDsl;
/*
// `DslBundle` requires `Default`
#[derive(Default)]
pub struct ImperiumDsl {
    inner: UiDsl,
    /// use custom text implementation to get around cuicui text not working
    f_text: Option<Box<str>>,
    f_text_color: Color,
    f_font_size: f32,
    f_font: Option<Handle<Font>>,
}

impl Default for ImperiumDsl {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            f_text: None,
            f_text_color: Color::WHITE,
            f_font_size: 12.0,
            font: None,
        }
    }
}
#[parse_dsl_impl(delegate = inner)]
impl ImperiumDsl {
    fn f_text(&mut self) {
        self.f_text = true;
    }
}


impl DslBundle for ImperiumDsl {
    fn insert(&mut self, cmds: &mut EntityCommands) -> Entity {
        let id = self.inner.insert(cmds);
        if let Some(text) = self.f_text.take() {
            let child_bundle = TextBundle::from_section(
                text,
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: self.f_font_size,
                    color: Color::WHITE,
                },
            )
            .with_style(Style {
                margin: UiRect::all(Val::Px(5.)),
                ..default()
            });
            cmds.with_children(|c| {
                c.spawn(child_bundle);
            });
        }
        id
    }
}*/