use bevy_ecs::{
    event::{Event, Events},
    system::ResMut,
    world::World,
};

pub fn init<T: Event>(world: &mut World) {
    world.insert_resource(Events::<T>::default());
}

pub fn update<T: Event>(mut events: ResMut<Events<T>>) {
    events.update();
}
