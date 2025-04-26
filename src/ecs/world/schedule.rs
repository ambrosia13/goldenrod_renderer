use bevy_ecs::{
    schedule::{IntoScheduleConfigs, Schedule, ScheduleLabel},
    world::World,
};

use crate::{
    app::{
        camera, fps, input, menu,
        object::{
            self, AabbPopEvent, AabbPushEvent, MaterialPopEvent, MaterialPushEvent, SpherePopEvent,
            SpherePushEvent, TrianglePopEvent, TrianglePushEvent,
        },
        renderer, time,
    },
    ecs::event,
    render::{
        shader::{self, ShaderRecompileEvent},
        WindowResizeEvent,
    },
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
        let mut init_main = Schedule::new(InitMainSchedule);
        init_main.add_systems((
            fps::FpsCounter::init,
            menu::Menu::init,
            object::Objects::init,
            object::binding::ObjectBinding::init,
            camera::Camera::init,
            camera::binding::ScreenBinding::init,
        ));

        let mut update_main = Schedule::new(UpdateMainSchedule);
        update_main.add_systems((
            menu::Menu::update,
            object::Objects::update,
            object::binding::ObjectBinding::update,
            camera::Camera::update,
            camera::binding::ScreenBinding::update,
        ));

        let mut post_update_main = Schedule::new(PostUpdateMainSchedule);
        post_update_main.add_systems((
            input::update_system,
            // Updating the fps counter comes before the time so we can get the most accurate time before the frame ends
            (time::update_system).chain(),
        ));

        let mut init_render = Schedule::new(InitRenderSchedule);
        init_render.add_systems(
            (
                renderer::profiler::RenderProfiler::init,
                renderer::pathtrace::PathtracePass::init,
                renderer::final_pass::FinalPass::init,
            )
                .chain(),
        );

        let mut pre_update_render = Schedule::new(PreUpdateRenderSchedule);
        pre_update_render.add_systems((
            shader::recompile_shaders,
            //texture::update_screen_size_textures,
        ));

        let mut update_render = Schedule::new(UpdateRenderSchedule);
        update_render.add_systems(
            (
                renderer::pathtrace::PathtracePass::update,
                renderer::final_pass::FinalPass::update,
            )
                .chain(),
        );

        let post_update_render = Schedule::new(PostUpdateRenderSchedule);

        let mut init_event = Schedule::new(InitEventSchedule);
        init_event.add_systems((
            event::init::<WindowResizeEvent>,
            event::init::<ShaderRecompileEvent>,
            event::init::<MaterialPushEvent>,
            event::init::<MaterialPopEvent>,
            event::init::<SpherePushEvent>,
            event::init::<SpherePopEvent>,
            event::init::<AabbPushEvent>,
            event::init::<AabbPopEvent>,
            event::init::<TrianglePushEvent>,
            event::init::<TrianglePopEvent>,
        ));

        let mut update_event = Schedule::new(UpdateEventSchedule);
        update_event.add_systems((
            event::update::<WindowResizeEvent>,
            event::update::<ShaderRecompileEvent>,
            event::update::<MaterialPushEvent>,
            event::update::<MaterialPopEvent>,
            event::update::<SpherePushEvent>,
            event::update::<SpherePopEvent>,
            event::update::<AabbPushEvent>,
            event::update::<AabbPopEvent>,
            event::update::<TrianglePushEvent>,
            event::update::<TrianglePopEvent>,
        ));

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
    pub fn startup(&mut self, world: &mut World) {
        self.init_main.run(world);
        self.init_render.run(world);
        self.init_event.run(world);
    }

    pub fn update(&mut self, world: &mut World) {
        self.update_main.run(world);
        self.post_update_main.run(world);
        self.update_event.run(world);

        self.pre_update_render.run(world);
        self.update_render.run(world);
        self.post_update_render.run(world);
    }
}
