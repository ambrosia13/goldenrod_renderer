use std::sync::Arc;

use bevy_ecs::{event::Events, world::World};
use glam::DVec2;
use input::Input;
use time::Time;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};

use crate::{
    ecs::{
        events::{KeyEvent, MenuResizeEvent, MouseInput, MouseMotion},
        schedule::Schedules,
        Wrapper,
    },
    egui::EguiRenderState,
    render::{FrameRecord, SurfaceState},
};

pub mod camera;
pub mod control;
pub mod fps;
pub mod input;
pub mod menu;
pub mod object;
pub mod renderer;
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
        schedules: Schedules,
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
        let mut schedules = Schedules::default();

        let surface_state = pollster::block_on(SurfaceState::new(window.clone()));
        let egui_render_state = EguiRenderState::new(
            &surface_state.gpu.device,
            surface_state.config.format,
            None,
            1,
            &window,
        );

        world.insert_resource(Wrapper::new(window.clone()));
        world.insert_resource(surface_state);
        world.insert_resource(egui_render_state);
        world.insert_resource(Input::new());
        world.insert_resource(Time::new());

        // Run startup systems
        schedules.on_init_event_setup.run(&mut world);
        schedules.on_init_render_setup.run(&mut world);
        schedules.on_init_app_setup.run(&mut world);

        Self::Init {
            window,
            world,
            schedules,
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

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let AppState::Init { world, .. } = &mut self.state else {
            return;
        };

        if let DeviceEvent::MouseMotion {
            delta: (delta_x, delta_y),
        } = event
        {
            world.send_event(MouseMotion(DVec2::new(delta_x, delta_y)));
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
            schedules,
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
                world.send_event(KeyEvent(event));
            }
            WindowEvent::MouseInput { state, button, .. } => {
                world.send_event(MouseInput { state, button });
            }
            WindowEvent::MouseWheel { delta: _, .. } => {}

            // Lifecycle events
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                let mut surface_state = world.resource_mut::<SurfaceState>();
                surface_state.resize(size);

                schedules.on_resize.run(world);
            }
            WindowEvent::RedrawRequested => {
                // We want another frame after this one
                window.request_redraw();

                // In the case of a menu resize, run the resize events
                let mut resize_events = world.resource_mut::<Events<MenuResizeEvent>>();
                if !resize_events.is_empty() {
                    resize_events.clear();
                    schedules.on_resize.run(world);
                }

                // Before starting the frame, run pre-frame systems
                schedules.on_redraw_pre_frame.run(world);

                // Initialize frame data
                let surface_state = world.resource::<SurfaceState>();
                let frame = match surface_state.begin_frame() {
                    Ok(r) => r,
                    Err(
                        wgpu::SurfaceError::Lost
                        | wgpu::SurfaceError::Outdated
                        | wgpu::SurfaceError::Other,
                    ) => {
                        surface_state.reconfigure_surface();
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
                let gpu_handle = surface_state.gpu.clone();
                let surface_texture_view = frame.surface_texture_view.clone();

                let mut egui_render_state = world.resource_mut::<EguiRenderState>();
                egui_render_state.begin_frame(window);

                // Pass over ownership of the frame data to the world for use in systems
                world.insert_resource(FrameRecord(frame));

                // Run render-related systems
                schedules.on_redraw_render.run(world);

                // Now that all render systems are done running, take frame data back out so we can use it to finish
                // drawing egui and then send all commands to the GPU
                let mut frame = world.remove_resource::<FrameRecord>().unwrap();

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

                // Notify the window that we're about to present, because apparently winit recommends that
                // https://docs.rs/winit/0.30.11/winit/window/struct.Window.html#method.pre_present_notify
                window.pre_present_notify();

                let surface_state = world.resource_mut::<SurfaceState>();
                surface_state.finish_frame(frame.0);

                // Now that the frame is done, run post-frame systems
                schedules.on_redraw_post_frame.run(world);
                schedules.on_redraw_event_update.run(world);
            }
            _ => {}
        }
    }
}
