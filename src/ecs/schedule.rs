use bevy_ecs::schedule::{IntoScheduleConfigs, Schedule, ScheduleLabel};

use crate::{
    app::{
        camera, control, fps, input, lookup, menu, object,
        renderer::{self, profiler},
        time,
    },
    ecs::{
        event,
        events::{KeyEvent, MenuResizeEvent, MouseInput, MouseMotion},
    },
};

/*
***App lifecycle***

Startup:
    OnInitEventSetup
    OnInitRenderSetup
    OnInitAppSetup

Per-frame:
    OnRedrawPreFrame
    OnRedrawRender
    OnRedrawPostFrame
    OnRedrawEventUpdate

Event-driven:
    OnResize
*/

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct OnResizeSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct OnInitEventSetupSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct OnInitRenderSetupSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct OnInitAppSetupSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct OnRedrawPreFrameSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct OnRedrawRenderSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct OnRedrawPostFrameSchedule;

#[derive(ScheduleLabel, Eq, PartialEq, Copy, Clone, Hash, Debug)]
struct OnRedrawEventUpdateSchedule;

pub struct Schedules {
    // startup schedules
    pub on_init_event_setup: Schedule,
    pub on_init_render_setup: Schedule,
    pub on_init_app_setup: Schedule,

    // per-frame schedules
    pub on_redraw_pre_frame: Schedule,
    pub on_redraw_render: Schedule,
    pub on_redraw_post_frame: Schedule,
    pub on_redraw_event_update: Schedule,

    // event-driven schedules
    pub on_resize: Schedule,
}

impl Default for Schedules {
    fn default() -> Self {
        // startup schedules
        let on_init_event_setup = Schedule::new(OnInitEventSetupSchedule);
        let on_init_render_setup = Schedule::new(OnInitRenderSetupSchedule);
        let on_init_app_setup = Schedule::new(OnInitAppSetupSchedule);

        // per-frame schedules
        let on_redraw_pre_frame = Schedule::new(OnRedrawPreFrameSchedule);
        let on_redraw_render = Schedule::new(OnRedrawRenderSchedule);
        let on_redraw_post_frame = Schedule::new(OnRedrawPostFrameSchedule);
        let on_redraw_event_update = Schedule::new(OnRedrawEventUpdateSchedule);

        // event-driven schedules
        let on_resize = Schedule::new(OnResizeSchedule);

        let mut schedules = Self {
            on_init_event_setup,
            on_init_render_setup,
            on_init_app_setup,
            on_redraw_pre_frame,
            on_redraw_render,
            on_redraw_post_frame,
            on_redraw_event_update,
            on_resize,
        };

        schedules.on_init_event_setup.add_systems((
            event::init::<MenuResizeEvent>,
            event::init::<MouseMotion>,
            event::init::<KeyEvent>,
            event::init::<MouseInput>,
        ));

        schedules.on_init_render_setup.add_systems(
            (
                (
                    renderer::RendererViewport::init,
                    lookup::SpectrumBinding::init,
                    lookup::CameraResponseBinding::init,
                    object::binding::ObjectBinding::init,
                    camera::binding::ScreenBinding::init,
                    renderer::profiler::RenderProfiler::init,
                ),
                (
                    (
                        renderer::material::MaterialTextures::init,
                        renderer::material::MaterialPipelines::init,
                    )
                        .chain(),
                    renderer::pathtrace::PathtracePass::init,
                    renderer::final_pass::FinalPass::init,
                )
                    .chain(),
            )
                .chain(),
        );

        schedules.on_init_app_setup.add_systems((
            fps::FpsCounter::init,
            menu::Menu::init,
            object::Objects::init,
            camera::Camera::init,
        ));

        schedules.on_redraw_pre_frame.add_systems(
            (
                // Input processing before everything else
                input::handle_keyboard_input_event,
                input::handle_mouse_input_event,
                (control::input_control, camera::Camera::update),
            )
                .chain(),
        );

        schedules.on_redraw_render.add_systems((
            menu::Menu::update,
            object::binding::ObjectBinding::update,
            camera::binding::ScreenBinding::update,
            (
                renderer::material::draw,
                renderer::pathtrace::PathtracePass::update,
                renderer::final_pass::FinalPass::update,
            )
                .chain(),
        ));

        schedules.on_redraw_post_frame.add_systems(
            (
                (
                    input::update_system,
                    time::update_system,
                    profiler::RenderProfiler::post_render,
                ),
                // Run the fps update after EVERYTHING is done
                fps::FpsCounter::update,
            )
                .chain(),
        );

        schedules.on_redraw_event_update.add_systems((
            event::update::<MenuResizeEvent>,
            event::update::<MouseMotion>,
            event::update::<KeyEvent>,
            event::update::<MouseInput>,
        ));

        schedules.on_resize.add_systems((
            camera::Camera::on_resize,
            renderer::material::MaterialTextures::on_resize,
            renderer::pathtrace::PathtracePass::on_resize,
            renderer::final_pass::FinalPass::on_resize,
        ));

        schedules
    }
}
