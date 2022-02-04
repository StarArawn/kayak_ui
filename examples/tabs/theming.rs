use kayak_ui::core::{rsx, widget, Children, Color, WidgetProps};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TabTheme {
    pub primary: Color,
    pub bg: Color,
    pub fg: Color,
    pub focus: Color,
    pub text: ColorState,
    pub active_tab: ColorState,
    pub inactive_tab: ColorState,
    pub tab_height: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ColorState {
    pub normal: Color,
    pub hovered: Color,
    pub active: Color,
}

#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct TabThemeProviderProps {
    pub initial_theme: TabTheme,
    #[prop_field(Children)]
    pub children: Option<Children>,
}

#[widget]
pub fn TabThemeProvider(props: TabThemeProviderProps) {
    let TabThemeProviderProps {
        initial_theme,
        children,
    } = props.clone();
    context.create_provider(initial_theme);
    rsx! { <>{children}</> }
}
