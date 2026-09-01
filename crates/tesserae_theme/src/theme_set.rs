use gpui::{App, Entity, Global};

use crate::{EntityWrapper, Theme, ThemeConfig, global_set_entity};

#[derive(Clone, Debug)]
pub struct ThemeSet {
    pub light: Theme,
    pub dark: Theme,
}

impl ThemeSet {
    pub fn generate(config: impl AsRef<ThemeConfig>) -> ThemeSet {
        let config = config.as_ref();

        Self {
            light: Theme::generate(config, ThemeSetKind::Light),
            dark: Theme::generate(config, ThemeSetKind::Dark),
        }
    }

    pub fn get_theme(&self, kind: ThemeSetKind) -> &Theme {
        match kind {
            ThemeSetKind::Light => &self.light,
            ThemeSetKind::Dark => &self.dark,
        }
    }

    pub fn set_global(cx: &mut App, theme_set: Self) {
        global_set_entity::<ThemeSetState, _>(cx, theme_set)
    }
}

#[derive(Clone)]
pub(crate) struct ThemeSetState(pub(crate) Entity<ThemeSet>);

impl Global for ThemeSetState {}

impl EntityWrapper<ThemeSet> for ThemeSetState {
    fn new(inner: Entity<ThemeSet>) -> Self {
        Self(inner)
    }

    fn entity(&self) -> &Entity<ThemeSet> {
        &self.0
    }
}

#[derive(Clone, Copy)]
pub enum ThemeSetKind {
    Light,
    Dark,
}

impl ThemeSetKind {
    pub fn set_global(cx: &mut App, kind: Self) {
        global_set_entity::<ThemeSetKindState, _>(cx, kind)
    }
}

#[derive(Clone)]
pub(crate) struct ThemeSetKindState(pub(crate) Entity<ThemeSetKind>);

impl Global for ThemeSetKindState {}

impl EntityWrapper<ThemeSetKind> for ThemeSetKindState {
    fn new(inner: Entity<ThemeSetKind>) -> Self {
        Self(inner)
    }

    fn entity(&self) -> &Entity<ThemeSetKind> {
        &self.0
    }
}
