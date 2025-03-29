use std::{
    collections::{LinkedList, VecDeque},
    sync::Arc,
};

use bevy_ecs::{
    event::EventWriter,
    system::{Res, ResMut, Resource},
    world::World,
};
use glam::Vec3;
use winit::{keyboard::KeyCode, window::Window};

use crate::{
    ecs::Wrapper,
    egui::EguiRenderState,
    render::{RenderState, WindowResizeEvent},
};

use super::{fps::FpsCounter, input::Input, renderer::profiler::RenderProfiler};

#[derive(Resource, Default)]
pub struct Menu {
    pub central_viewport_start: (u32, u32),
    pub central_viewport_end: (u32, u32),

    // use linked list because there is almost always only one element, and a Vec would require reallocating the
    // compile error strings
    pub shader_compile_errors: LinkedList<String>,

    pub settings: Settings,
}

impl Menu {
    pub fn init(world: &mut World) {
        let render_state = world.resource::<RenderState>();

        let menu = Menu {
            central_viewport_start: (0, 0),
            central_viewport_end: (render_state.size.width, render_state.size.height),
            ..Default::default()
        };

        world.insert_resource(menu);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        mut menu: ResMut<Menu>,
        mut render_state: ResMut<RenderState>,
        egui_render_state: Res<EguiRenderState>,
        window: Res<Wrapper<Arc<Window>>>,
        input: Res<Input>,
        fps_counter: Res<FpsCounter>,
        profiler: Res<RenderProfiler>,
        mut resize_events: EventWriter<WindowResizeEvent>,
    ) {
        let pixels_per_point = egui_render_state.context().pixels_per_point();

        let mut central_panel_offset_from_left = 0.0;
        let mut central_panel_offset_from_right = 0.0;

        // When the user presses escape, disable fullscreen
        if input.keys.just_pressed(KeyCode::Escape) {
            menu.settings.fullscreen = false;
        }

        // Skip drawing the menu if fullscreen is enabled
        if !menu.settings.fullscreen {
            egui::SidePanel::left("left")
                .resizable(true)
                .default_width(240.0)
                .min_width(80.0)
                .show(egui_render_state.context(), |ui| {
                    central_panel_offset_from_left = ui.available_width() * pixels_per_point;

                    ui.heading("General");

                    let fullscreen_button = ui.button("Fullscreen");

                    if fullscreen_button.hovered() {
                        fullscreen_button.show_tooltip_text("Press ESC to open the menus again");
                    }

                    menu.settings.fullscreen = fullscreen_button.clicked();

                    ui.separator();

                    ui.heading("Path tracing");
                    ui.checkbox(&mut menu.settings.accumulate, "Enable accumulation");
                    ui.checkbox(&mut menu.settings.spectral, "Spectral rendering");
                });

            egui::SidePanel::right("right")
                .resizable(true)
                .default_width(160.0)
                .min_width(80.0)
                .show(egui_render_state.context(), |ui| {
                    central_panel_offset_from_right = ui.available_width() * pixels_per_point;

                    ui.heading("Debug");

                    ui.label(format!("FPS: {:.1}", fps_counter.average_fps()));

                    for (name, time) in profiler.times.iter() {
                        ui.label(format!("{}: {:.3} ms", name, time.as_secs_f64() * 1000.0));
                    }

                    ui.separator();

                    ui.heading("Camera");
                });

            // If there was a shader compile error, display it to the screen
            for error in menu.shader_compile_errors.iter() {
                egui::Window::new("Shader compile error").show(egui_render_state.context(), |ui| {
                    ui.label(error);
                });
            }
        }

        let window_size = window.inner_size();

        // Calculate the bounds of the central panel, so our renderer knows how big the texture we draw to should be
        let effective_viewport_start = (central_panel_offset_from_left as u32, 0);
        let effective_viewport_end = (
            window_size.width - central_panel_offset_from_right as u32,
            window_size.height,
        );

        // if it's changed, send a resize event
        if render_state.effective_viewport_start != effective_viewport_start
            || render_state.effective_viewport_end != effective_viewport_end
        {
            resize_events.send(WindowResizeEvent);
        }

        render_state.effective_viewport_start = effective_viewport_start;
        render_state.effective_viewport_end = effective_viewport_end;
    }
}

pub struct Settings {
    pub fullscreen: bool,
    pub accumulate: bool,
    pub spectral: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            accumulate: true,
            spectral: true,
        }
    }
}
