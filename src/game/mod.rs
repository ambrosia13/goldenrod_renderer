use std::sync::Arc;

use bevy_ecs::world::World;
use input::Input;
use time::Time;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};

use crate::{
    ecs::{world::schedule::ScheduleRunner, Wrapper},
    render::{FrameData, RenderState, WindowResizeEvent},
    ui::EguiRenderState,
};

pub mod input;
pub mod time;

pub fn run() {
    let event_loop = EventLoop::new().expect("Couldn't create window event loop");
    let mut app = App {
        state: AppState::Uninit,
    };

    event_loop.run_app(&mut app).unwrap();
}

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
    pub fn is_uninit(&self) -> bool {
        matches!(self, AppState::Uninit)
    }

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
        let mut schedule_runner = ScheduleRunner::default();

        let render_state = pollster::block_on(RenderState::new(window.clone()));
        let egui_render_state = EguiRenderState::new(
            &render_state.gpu_handle.device,
            render_state.config.format,
            None,
            1,
            &window,
        );

        world.insert_resource(Wrapper::new(window.clone()));
        world.insert_resource(render_state);
        world.insert_resource(egui_render_state);
        world.insert_resource(Input::new());
        world.insert_resource(Time::new());

        // Run startup systems
        schedule_runner.startup(&mut world);

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
        if self.state.is_uninit() {
            self.state = AppState::init(event_loop);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let AppState::Init {
            window,
            world,
            schedule_runner,
        } = &mut self.state
        else {
            return;
        };

        if window.id() != window_id {
            return;
        }

        // Let the egui context process the update on its own
        let mut egui_render_state = world.resource_mut::<EguiRenderState>();
        egui_render_state.handle_input(window, &event);

        // Now our app will process event
        match event {
            // Input events
            WindowEvent::KeyboardInput { event, .. } => {
                let mut input = world.resource_mut::<Input>();
                input::handle_keyboard_input_event(&mut input, event);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let mut input = world.resource_mut::<Input>();
                input::handle_mouse_input_event(&mut input, state, button);
            }
            WindowEvent::MouseWheel { delta: _, .. } => {}

            // Lifecycle events
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                let mut render_state = world.resource_mut::<RenderState>();
                render_state.resize(size);

                world.send_event(WindowResizeEvent(size));
            }
            WindowEvent::RedrawRequested => {
                // We want another frame after this one
                window.request_redraw();

                let render_state = world.resource::<RenderState>();
                let frame = match render_state.begin_frame() {
                    Ok(r) => r,
                    Err(
                        wgpu::SurfaceError::Lost
                        | wgpu::SurfaceError::Outdated
                        | wgpu::SurfaceError::Other,
                    ) => {
                        render_state.reconfigure_surface();
                        return;
                    }
                    Err(wgpu::SurfaceError::Timeout) => {
                        log::warn!("Surface timeout");
                        return;
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        log::error!("Out of memory, exiting");
                        event_loop.exit();
                        return;
                    }
                };

                // We need access to the device, queue, and a view of the surface texture in order to draw egui,
                // so get those from the render state before dropping it and passing over ownership of the frame data
                let gpu_handle = render_state.gpu_handle.clone();
                let surface_texture_view = frame
                    .surface_texture
                    .texture
                    .create_view(&Default::default());
                let frame_count = world.resource::<Time>().frame_count();

                let mut egui_render_state = world.resource_mut::<EguiRenderState>();
                egui_render_state.begin_frame(window);

                egui::Window::new("goldenrod renderer").show(egui_render_state.context(), |ui| {
                    if ui.button("Click me").clicked() {
                        log::info!("Button clicked");
                    }

                    ui.label(format!("Frames: {}k", frame_count / 1000));
                });

                // Pass over ownership of the frame data to the world, to let all the systems freely use it
                world.insert_resource(frame);
                schedule_runner.update(world);

                // Now that all systems are done running, take frame data back out so we can use it to finish
                // drawing egui and then send all commands to the GPU
                let mut frame = world.remove_resource::<FrameData>().unwrap();

                let mut egui_render_state = world.resource_mut::<EguiRenderState>();
                egui_render_state.end_frame_and_draw(
                    &gpu_handle.device,
                    &gpu_handle.queue,
                    &mut frame.encoder,
                    window,
                    &surface_texture_view,
                    egui_wgpu::ScreenDescriptor {
                        size_in_pixels: [window.inner_size().width, window.inner_size().height],
                        pixels_per_point: window.scale_factor() as f32,
                    },
                );

                let render_state = world.resource_mut::<RenderState>();
                render_state.finish_frame(frame);
            }
            _ => {}
        }
    }
}
