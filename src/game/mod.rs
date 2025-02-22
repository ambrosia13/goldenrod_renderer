use std::sync::Arc;

use bevy_ecs::world::World;
use winit::{
    application::ApplicationHandler,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

use crate::{
    ecs::{world::schedule::ScheduleRunner, Wrapper},
    window,
};

pub fn run() {}

#[allow(clippy::large_enum_variant)]
enum AppState {
    Uninit,
    Init {
        window: Arc<Window>,
        world: World,
        schedule_runner: ScheduleRunner,
    },
}

impl AppState {
    pub fn init(event_loop: &ActiveEventLoop) -> Self {
        let window_attributes = WindowAttributes::default()
            .with_title("goldenrod renderer")
            .with_maximized(true);

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("Couldn't create window"),
        );

        let mut world = World::new();
        let schedule_runner = ScheduleRunner::default();

        world.insert_resource(Wrapper::new(window.clone()));

        Self::Init {
            window,
            world,
            schedule_runner,
        }
    }
}

pub struct App {
    state: AppState,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if matches!(self.state, AppState::Uninit) {
            self.state = AppState::init(event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        todo!()
    }
}
