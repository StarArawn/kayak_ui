use bevy::{
    input::{
        keyboard::KeyboardInput,
        mouse::{MouseButtonInput, MouseScrollUnit, MouseWheel},
        ButtonState,
    },
    prelude::*,
};

use crate::{
    context::{CustomEventReader, KayakRootContext},
    event_dispatcher::EventDispatcher,
    input_event::InputEvent,
};

pub(crate) fn process_events(world: &mut World) {
    let mut input_events = Vec::new();

    query_world::<
        (
            Res<Events<CursorMoved>>,
            Res<Events<MouseButtonInput>>,
            Res<Events<MouseWheel>>,
            Res<Events<ReceivedCharacter>>,
            Res<Events<KeyboardInput>>,
            ResMut<CustomEventReader<CursorMoved>>,
            ResMut<CustomEventReader<MouseButtonInput>>,
            ResMut<CustomEventReader<MouseWheel>>,
            ResMut<CustomEventReader<ReceivedCharacter>>,
            ResMut<CustomEventReader<KeyboardInput>>,
        ),
        _,
        _,
    >(
        |(
            cursor_moved_events,
            mouse_button_input_events,
            mouse_wheel_events,
            char_input_events,
            keyboard_input_events,
            mut custom_event_reader_cursor,
            mut custom_event_mouse_button,
            mut custom_event_mouse_wheel,
            mut custom_event_char_input,
            mut custom_event_keyboard,
        )| {
            if let Some(event) = custom_event_reader_cursor
                .0
                .read(&cursor_moved_events)
                .last()
            {
                // Currently, we can only handle a single MouseMoved event at a time so everything but the last needs to be skipped
                input_events.push(InputEvent::MouseMoved(event.position.into()));
            }

            for event in custom_event_mouse_button.0.read(&mouse_button_input_events) {
                if let MouseButton::Left = event.button {
                    if event.state == ButtonState::Pressed {
                        input_events.push(InputEvent::MouseLeftPress);
                    } else if event.state == ButtonState::Released {
                        input_events.push(InputEvent::MouseLeftRelease);
                    }
                }
            }

            for MouseWheel {
                x,
                y,
                unit,
                window: _,
            } in custom_event_mouse_wheel.0.read(&mouse_wheel_events)
            {
                input_events.push(InputEvent::Scroll {
                    dx: *x,
                    dy: *y,
                    is_line: matches!(unit, MouseScrollUnit::Line),
                })
            }

            for event in custom_event_char_input.0.read(&char_input_events) {
                input_events.push(InputEvent::CharEvent {
                    c: event.char.clone(),
                });
            }

            for event in custom_event_keyboard.0.read(&keyboard_input_events) {
                input_events.push(InputEvent::Keyboard {
                    key: event.key_code,
                    is_pressed: matches!(event.state, ButtonState::Pressed),
                });
            }
        },
        world,
    );

    // TODO: find a faster way of doing this.
    let mut context_data = Vec::new();

    query_world::<Query<(Entity, &mut EventDispatcher, &mut KayakRootContext)>, _, _>(
        |mut query| {
            for (entity, mut event_dispatcher, mut kayak_root_context) in query.iter_mut() {
                context_data.push((
                    entity,
                    std::mem::take(&mut *event_dispatcher),
                    std::mem::take(&mut *kayak_root_context),
                ));
            }
        },
        world,
    );

    for (entity, mut event_dispatcher, mut context) in context_data.drain(..) {
        event_dispatcher.process_events(&input_events, &mut context, world);

        world.entity_mut(entity).insert((event_dispatcher, context));
    }
}

pub(crate) fn query_world<T: bevy::ecs::system::SystemParam + 'static, F, R>(
    f: F,
    world: &mut World,
) -> R
where
    F: FnOnce(<T as bevy::ecs::system::SystemParam>::Item<'_, '_>) -> R,
{
    let mut system_state = bevy::ecs::system::SystemState::<T>::new(world);
    let r = {
        let test = system_state.get_mut(world);
        f(test)
    };
    system_state.apply(world);

    r
}
