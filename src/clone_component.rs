use bevy::{ecs::system::CommandQueue, prelude::*};

use crate::widget_state::WidgetState;

#[derive(Component, Default)]
pub struct PreviousWidget;

#[derive(Default)]
pub(crate) struct EntityCloneSystems(
    pub  Vec<(
        fn(&mut World, Entity, Entity),
        fn(&mut World, Entity, Entity, &WidgetState),
    )>,
);

pub(crate) fn clone_system<T: Clone + Component>(
    world: &mut World,
    target: Entity,
    reference: Entity,
) {
    if let Some(v) = world.entity(reference).get::<T>() {
        let v = v.clone();
        world.entity_mut(target).insert(v);
    }
}

pub(crate) fn clone_state<State: Component + PartialEq + Clone>(
    world: &mut World,
    target: Entity,
    reference: Entity,
    widget_state: &WidgetState,
) {
    if let Some(reference_state_entity) = widget_state.get(reference) {
        if let Some(v) = world.entity(reference_state_entity).get::<State>() {
            if let Some(target_state_entity) = widget_state.get(target) {
                let v = v.clone();
                world.entity_mut(target_state_entity).insert(v);
            } else {
                let mut command_queue = CommandQueue::default();
                let mut commands = Commands::new(&mut command_queue, world);
                let state_entity = widget_state.add::<State>(&mut commands, target, v.clone());
                commands.entity(state_entity).insert(PreviousWidget);
                command_queue.apply(world);
            }
        }
    }
}
