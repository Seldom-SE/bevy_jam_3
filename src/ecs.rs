use bevy::ecs::{system::EntityCommands, world::EntityMut};

use crate::prelude::*;

pub trait DynBundle {
    fn spawn<'w, 's, 'a>(
        // Obscure syntax to the rescue!
        self: Box<Self>,
        commands: &'a mut Commands<'w, 's>,
    ) -> EntityCommands<'w, 's, 'a>;
    fn world_spawn(self: Box<Self>, world: &mut World) -> EntityMut;
}

impl<T: Bundle> DynBundle for T {
    fn spawn<'w, 's, 'a>(
        self: Box<Self>,
        commands: &'a mut Commands<'w, 's>,
    ) -> EntityCommands<'w, 's, 'a> {
        commands.spawn(*self)
    }

    fn world_spawn(self: Box<Self>, world: &mut World) -> EntityMut {
        world.spawn(*self)
    }
}
