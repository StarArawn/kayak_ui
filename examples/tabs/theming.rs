use kayak_ui::{core::{Color, rsx, widget}};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TabTheme {
    pub primary: Color,
    pub bg: Color,
    pub fg: Color,
    pub focus: Color,
    pub text: ColorState,
    pub active_tab: ColorState,
    pub inactive_tab: ColorState,
    pub tab_height: f32
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ColorState {
    pub normal: Color,
    pub hovered: Color,
    pub active: Color,
}

#[widget]
pub fn TabThemeProvider(initial_theme: TabTheme) {
    context.create_provider(initial_theme);
    rsx! { <>{children}</> }
}