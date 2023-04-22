# Chapter 3 - Creating custom widgets!
Kayak UI allows users to create custom widgets.

Widgets are structured in a few different ways:
1. Widgets at a bare minimum must include a Props component, Widget Bundle, and the update and render systems.
2. Widgets can include custom components and data but the default `widget_update` system only supports diffing of props and state.
3. Widgets have some base components which are auto checked these are: KStyles and KChildren.

I think it's best if we showcase a simple example:
```rust
// At a bare minimum the widget props component must include these derives. 
// This is because we need to diff the previous values of these.
// Default is used to make creating widgets a little easier.
// And component is required since this is a bevy component!
#[derive(Component, Clone, PartialEq, Default)]
pub struct MyButtonProps {

}

// In the future this will tell Kayak that these Props belongs to a widget.
// For now it's use to get the `WidgetName` component.
impl Widget for MyButtonProps { }

// Now we need a widget bundle this can represent a collection of components our widget might have
// Note: You can include custom data here. Just don't expect it to get diffed during update!
#[derive(Bundle)]
pub struct MyButtonBundle {
    pub props: MyButtonProps,
    pub styles: KStyle,
    pub computed_styles: ComputedStyles,
    pub children: KChildren,
    // This allows us to hook into on click events!
    pub on_event: OnEvent,
    // Widget name is required by Kayak UI!
    pub widget_name: WidgetName,
}

impl Default for MyButtonBundle {
    fn default() -> Self {
        Self {
            props: MyButtonProps::default(),
            styles: KStyle::default(),
            computed_styles: ComputedStyles::default(),
            children: KChildren::default(),
            on_event: OnEvent::default(),
            // Kayak uses this component to find out more information about your widget.
            // This is done because bevy does not have the ability to query traits.
            widget_name: MyButtonProps::default().get_name(),
        }
    }
}

// Now we need to create our systems.
// Since our button doesn't have any custom data we can diff using the default widget_update system.
// We do need to create a render system!

pub fn my_button_render(
    // This is a bevy feature which allows custom parameters to be passed into a system.
    // In this case Kayak UI gives the system an `Entity`.
    In(entity): In<Entity>,
    // This struct allows us to make changes to the widget tree.
    mut widget_context: ResMut<KayakWidgetContext>,
    // The rest of the parameters are just like those found in a bevy system!
    // In fact you can add whatever you would like here including more queries or lookups
    // to resources within bevy's ECS.
    mut commands: Commands,
    // In this case we really only care about our buttons children! Let's query for them.
    mut query: Query<&KChildren>,
) -> bool {
    // Grab our children for our button widget:
    if let Ok(children) = query.get(entity) {

        let background_styles = KStyle {
            // Lets use red for our button background!
            background_color: StyleProp::Value(Color::RED),
            // 50 pixel border radius.
            border_radius: Corner::all(50.0).into(),
            ..Default::default()
        };

        let parent_id = Some(entity);

        rsx! {
            <BackgroundBundle
                styles={background_styles}
                // We pass the children to the background bundle!
                children={children.clone()}
            />
        };
    }

    // The boolean returned here tells kayak UI to update the tree. You can avoid tree updates by
    // returning false, but in practice this should be done rarely. As kayak diff's the tree and
    // will avoid tree updates if nothing has changed! 
    true
}

// Finally we need to let the core widget context know about our new widget!
fn startup(...) {

    // Default kayak startup stuff.
    ...

    // We need to register the prop and state types.
    // State is empty so you can use the `EmptyState` component!
    widget_context.add_widget_data::<MyButtonProps, EmptyState>();

    // Next we need to add the systems
    widget_context.add_widget_system(
        // We are registering these systems with a specific WidgetName.
        MyButtonProps::default().get_name(),
        // widget_update auto diffs props and state.
        // Optionally if you have context you can use: widget_update_with_context
        // otherwise you will need to create your own widget update system!
        widget_update::<MyButtonProps, EmptyState>,
        // Add our render system!
        my_button_render,
    );

    // We can now create our widget like:
    rsx! {
        <KayakAppBundle>
            <MyButtonBundle>
                <TextWidgetBundle
                    text={TextProps {
                        content: "Click me!".into(),
                        ..Default::default()
                    }}
                />
            </MyButtonBundle>
        </KayakAppBundle>
    } 
} 
```
