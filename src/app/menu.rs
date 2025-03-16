use std::sync::Arc;

use bevy_ecs::{
    system::{Res, ResMut, Resource},
    world::World,
};
use winit::{keyboard::KeyCode, window::Window};

use crate::{ecs::Wrapper, egui::EguiRenderState};

use super::{fps::FpsCounter, input::Input};

#[derive(Resource, Default)]
pub struct Menu {
    pub central_viewport_start: Option<(u32, u32)>,
    pub central_viewport_end: Option<(u32, u32)>,

    pub settings: Settings,
}

impl Menu {
    pub fn init(world: &mut World) {
        world.insert_resource(Menu::default());
    }

    pub fn update(
        mut menu: ResMut<Menu>,
        egui_render_state: Res<EguiRenderState>,
        window: Res<Wrapper<Arc<Window>>>,
        input: Res<Input>,
        fps_counter: Res<FpsCounter>,
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

                    // Fullscreen
                    menu.settings.fullscreen = ui.button("Fullscreen").clicked();

                    ui.separator();

                    ui.heading("Path tracing");

                    // Accumulation
                    ui.checkbox(&mut menu.settings.accumulate, "Enable Accumulation");
                });

            egui::SidePanel::right("right")
                .resizable(true)
                .default_width(160.0)
                .min_width(80.0)
                .show(egui_render_state.context(), |ui| {
                    central_panel_offset_from_right = ui.available_width() * pixels_per_point;

                    ui.heading("Debug");

                    ui.label(format!("FPS: {:.1}", fps_counter.average_fps()));

                    ui.separator();

                    ui.heading("Camera");
                });
        }

        let window_size = window.inner_size();

        // Calculate the bounds of the central panel, so our renderer knows how big the texture we draw to should be
        menu.central_viewport_start = Some((central_panel_offset_from_left as u32, 0));
        menu.central_viewport_end = Some((
            window_size.width - central_panel_offset_from_right as u32,
            window_size.height,
        ));
    }
}

pub struct Settings {
    pub fullscreen: bool,
    pub accumulate: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            accumulate: true,
        }
    }
}
