use bevy_ecs::{
    schedule::{Schedule, ScheduleLabel},
    world::World,
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
    update_event: Schedule,
}

impl Default for ScheduleRunner {
    fn default() -> Self {
        let init_render = Schedule::new(InitRenderSchedule);

        let init_main = Schedule::new(InitMainSchedule);

        let init_event = Schedule::new(InitEventSchedule);

        let pre_update_render = Schedule::new(PreUpdateRenderSchedule);

        let update_render = Schedule::new(UpdateRenderSchedule);

        let post_update_render = Schedule::new(PostUpdateRenderSchedule);

        let update_main = Schedule::new(UpdateMainSchedule);

        let update_event = Schedule::new(UpdateEventSchedule);

        Self {
            init_render,
            init_main,
            init_event,
            pre_update_render,
            update_render,
            post_update_render,
            update_main,
            update_event,
        }
    }
}

impl ScheduleRunner {
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
        self.update_event.run(world);
    }
}
