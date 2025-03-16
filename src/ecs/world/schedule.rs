use bevy_ecs::{
    schedule::{Schedule, ScheduleLabel},
    world::World,
};

use crate::{
    app::{input, time},
    ecs::event,
    render::WindowResizeEvent,
};

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct InitRenderSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct InitMainSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct InitEventSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct PreUpdateRenderSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct UpdateRenderSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct PostUpdateRenderSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct UpdateMainSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct PostUpdateMainSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct UpdateEventSchedule;

pub struct ScheduleRunner {
    // Startup schedules
    init_render: Schedule,
    init_main: Schedule,
    init_event: Schedule,

    // Update schedules
    pre_update_render: Schedule,
    update_render: Schedule,
    post_update_render: Schedule,
    update_main: Schedule,
    post_update_main: Schedule,
    update_event: Schedule,
}

impl Default for ScheduleRunner {
    fn default() -> Self {
        let init_render = Schedule::new(InitRenderSchedule);

        let init_main = Schedule::new(InitMainSchedule);

        let mut init_event = Schedule::new(InitEventSchedule);

        init_event.add_systems(event::init::<WindowResizeEvent>);

        let pre_update_render = Schedule::new(PreUpdateRenderSchedule);

        let update_render = Schedule::new(UpdateRenderSchedule);

        let post_update_render = Schedule::new(PostUpdateRenderSchedule);

        let update_main = Schedule::new(UpdateMainSchedule);

        let mut post_update_main = Schedule::new(PostUpdateMainSchedule);

        post_update_main.add_systems((input::update_system, time::update_system));

        let mut update_event = Schedule::new(UpdateEventSchedule);

        update_event.add_systems(event::update::<WindowResizeEvent>);

        Self {
            init_render,
            init_main,
            init_event,
            pre_update_render,
            update_render,
            post_update_render,
            update_main,
            post_update_main,
            update_event,
        }
    }
}

impl ScheduleRunner {
    pub fn add_observers(world: &mut World) {}

    pub fn startup(&mut self, world: &mut World) {
        self.init_render.run(world);
        self.init_main.run(world);
        self.init_event.run(world);
    }

    pub fn update(&mut self, world: &mut World) {
        self.pre_update_render.run(world);
        self.update_render.run(world);
        self.post_update_render.run(world);

        self.update_main.run(world);
        self.post_update_main.run(world);
        self.update_event.run(world);
    }
}
