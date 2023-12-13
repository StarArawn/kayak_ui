use crate::{
    context::WidgetName,
    styles::{ComputedStyles, KStyle, RenderCommand},
    widget::Widget,
};
use bevy::prelude::{Bundle, Component, Entity, Handle, In, Query};

pub use bevy_svg::prelude::Svg;
/// Renders a svg asset within the GUI
/// The rendered svg respects some of the styles.
#[derive(Component, PartialEq, Eq, Clone, Default)]
pub struct KSvg(pub Handle<Svg>);

impl Widget for KSvg {}

#[derive(Bundle)]
pub struct KSvgBundle {
    pub svg: KSvg,
    pub styles: KStyle,
    pub computed_styles: ComputedStyles,
    pub widget_name: WidgetName,
}

impl Default for KSvgBundle {
    fn default() -> Self {
        Self {
            svg: Default::default(),
            styles: Default::default(),
            computed_styles: ComputedStyles::default(),
            widget_name: KSvg::default().get_name(),
        }
    }
}

pub fn svg_render(
    In(entity): In<Entity>,
    mut query: Query<(&KStyle, &mut ComputedStyles, &KSvg)>,
) -> bool {
    if let Ok((style, mut computed_styles, svg)) = query.get_mut(entity) {
        *computed_styles = KStyle::default()
            .with_style(KStyle {
                render_command: RenderCommand::Svg {
                    handle: svg.0.clone_weak(),
                }
                .into(),
                ..Default::default()
            })
            .with_style(style)
            .into();
    }
    true
}
