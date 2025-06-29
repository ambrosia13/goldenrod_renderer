use std::sync::Arc;

use bevy_ecs::{
    event::EventWriter,
    resource::Resource,
    system::{Res, ResMut},
    world::World,
};
use egui::{DragValue, Ui};
use glam::Vec3;
use winit::window::Window;

use crate::{
    ecs::{events::MenuResizeEvent, Wrapper},
    egui::EguiRenderState,
    render::SurfaceState,
};

use super::{
    camera::Camera,
    fps::FpsCounter,
    object::{Aabb, Material, MaterialType, Objects, Sphere, Triangle},
    renderer::profiler::RenderProfiler,
};

fn vec3_editor(value: &mut Vec3, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label("X");
        ui.add(DragValue::new(&mut value.x).fixed_decimals(2).speed(0.01));

        ui.label("Y");
        ui.add(DragValue::new(&mut value.y).fixed_decimals(2).speed(0.01));

        ui.label("Z");
        ui.add(DragValue::new(&mut value.z).fixed_decimals(2).speed(0.01));
    });
}

#[derive(Resource, Default)]
pub struct Menu {
    pub settings: Settings,
    object_editor: ObjectEditor,
}

impl Menu {
    pub fn init(world: &mut World) {
        let menu = Menu::default();
        world.insert_resource(menu);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        mut menu: ResMut<Menu>,
        mut surface_state: ResMut<SurfaceState>,
        egui_render_state: Res<EguiRenderState>,
        window: Res<Wrapper<Arc<Window>>>,
        fps_counter: Res<FpsCounter>,
        camera: Res<Camera>,
        mut objects: ResMut<Objects>,
        profiler: Res<RenderProfiler>,
        mut resize_events: EventWriter<MenuResizeEvent>,
    ) {
        let mut left_panel_size = 0.0;
        let mut right_panel_size = 0.0;

        // Skip drawing the menu if fullscreen is enabled
        if !menu.settings.fullscreen {
            // Left panel is the control panel
            left_panel_size = menu.control_panel(egui_render_state.context(), &mut objects);

            // Right panel is the debug panel
            right_panel_size = menu.debug_panel(
                egui_render_state.context(),
                &fps_counter,
                &profiler,
                &camera,
                &objects,
            );
        }

        let window_size = window.inner_size();

        // Calculate the bounds of the central panel, so our renderer knows how big the texture we draw to should be
        let effective_viewport_start = (left_panel_size as u32, 0);
        let effective_viewport_end = (
            window_size.width - right_panel_size as u32,
            window_size.height,
        );

        // if it's changed, send a resize event
        if surface_state.effective_viewport_start != effective_viewport_start
            || surface_state.effective_viewport_end != effective_viewport_end
        {
            resize_events.write(MenuResizeEvent);
        }

        surface_state.effective_viewport_start = effective_viewport_start;
        surface_state.effective_viewport_end = effective_viewport_end;
    }

    // We need to take objects as a ResMut<> to preserve change detection
    // returns the size of the control panel
    fn control_panel(&mut self, ctx: &egui::Context, objects: &mut ResMut<Objects>) -> f32 {
        let mut panel_size = 0.0;

        egui::SidePanel::left("control_panel")
            .resizable(true)
            .default_width(240.0)
            .min_width(80.0)
            .show(ctx, |ui| {
                panel_size = ui.available_width() * ctx.pixels_per_point();

                ui.heading("General");

                let fullscreen_button = ui.button("Fullscreen");

                if fullscreen_button.hovered() {
                    fullscreen_button.show_tooltip_text("Press ESC to open the menus again");
                }

                self.settings.fullscreen = fullscreen_button.clicked();

                ui.separator();

                ui.heading("Path tracing");
                ui.checkbox(&mut self.settings.accumulate, "Enable accumulation");
                ui.checkbox(&mut self.settings.spectral, "Spectral rendering");

                ui.separator();

                ui.heading("Object Editor");

                ui.collapsing("Material", |ui| {
                    let material = &mut self.object_editor.material;

                    let mut albedo: [f32; 3] = material.albedo.into();
                    ui.horizontal(|ui| {
                        ui.label("Albedo");
                        ui.color_edit_button_rgb(&mut albedo);
                    });
                    material.albedo = albedo.into();

                    ui.horizontal(|ui| {
                        ui.label("Emission");
                        ui.add(
                            DragValue::new(&mut material.emission)
                                .speed(0.1)
                                .fixed_decimals(2)
                                .range(0.0..=f32::INFINITY),
                        );
                    });

                    ui.label("Material Type");
                    egui::ComboBox::from_label("")
                        .selected_text(format!("{:?}", material.ty))
                        .show_ui(ui, |ui| {
                            let material = &mut self.object_editor.material;

                            ui.selectable_value(
                                &mut material.ty,
                                MaterialType::Lambertian,
                                "Lambertian",
                            );
                            ui.selectable_value(&mut material.ty, MaterialType::Metal, "Metallic");
                            ui.selectable_value(
                                &mut material.ty,
                                MaterialType::Dielectric,
                                "Dielectric",
                            );
                        });

                    let show_roughness = matches!(
                        self.object_editor.material.ty,
                        MaterialType::Metal | MaterialType::Dielectric
                    );

                    let show_ior =
                        matches!(self.object_editor.material.ty, MaterialType::Dielectric);

                    if show_roughness {
                        ui.label("Roughness");
                        ui.add(egui::Slider::new(
                            &mut self.object_editor.material.roughness,
                            0.0..=1.0,
                        ));
                    }

                    if show_ior {
                        ui.label("Refractive Index");
                        ui.add(egui::Slider::new(
                            &mut self.object_editor.material.ior,
                            0.0..=4.0,
                        ));
                    }
                });

                ui.collapsing("Geometry", |ui| {
                    let geo_type = &mut self.object_editor.geometry_type;

                    egui::ComboBox::from_label("")
                        .selected_text(format!("{:?}", geo_type))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(geo_type, GeometryType::Sphere, "Sphere");
                            ui.selectable_value(geo_type, GeometryType::Aabb, "Box");
                            ui.selectable_value(geo_type, GeometryType::Triangle, "Triangle");
                        });

                    match geo_type {
                        GeometryType::Sphere => {
                            let sphere = &mut self.object_editor.sphere;

                            ui.label("Center");
                            vec3_editor(&mut sphere.center, ui);

                            ui.horizontal(|ui| {
                                ui.label("Radius");
                                ui.add(
                                    DragValue::new(&mut sphere.radius)
                                        .fixed_decimals(2)
                                        .speed(0.1)
                                        .range(0.01..=f32::INFINITY),
                                );
                            });
                        }
                        GeometryType::Aabb => {
                            let aabb = &mut self.object_editor.aabb;

                            ui.label("Start Bounds");
                            vec3_editor(&mut aabb.min, ui);

                            ui.label("End Bounds");
                            vec3_editor(&mut aabb.max, ui);

                            aabb.max = aabb.max.max(aabb.min + 0.01);
                        }
                        GeometryType::Triangle => {
                            let triangle = &mut self.object_editor.triangle;

                            ui.label("Vertices");
                            vec3_editor(&mut triangle.a, ui);
                            vec3_editor(&mut triangle.b, ui);
                            vec3_editor(&mut triangle.c, ui);
                        }
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        let mut reuse_material = false;

                        if let Some(prev_material) = objects.materials.last() {
                            if self.object_editor.material == *prev_material {
                                // To preserve memory, if the material hasn't changed, reuse it with the same index
                                reuse_material = true;
                            }
                        }

                        let material_index = if reuse_material {
                            objects.materials.len() as u32 - 1
                        } else {
                            objects.materials.push(self.object_editor.material);
                            objects.materials.len() as u32
                        };

                        match self.object_editor.geometry_type {
                            GeometryType::Sphere => {
                                self.object_editor.sphere.material_index = material_index;
                                objects.spheres.push(self.object_editor.sphere);
                            }
                            GeometryType::Aabb => {
                                self.object_editor.aabb.material_index = material_index;
                                objects.aabbs.push(self.object_editor.aabb);
                            }
                            GeometryType::Triangle => {
                                self.object_editor.triangle.material_index = material_index;
                                objects.triangles.push(self.object_editor.triangle);
                            }
                        }
                    }

                    if ui.button("Delete Last").clicked() {
                        match self.object_editor.geometry_type {
                            GeometryType::Sphere => {
                                objects.spheres.pop();
                            }
                            GeometryType::Aabb => {
                                objects.aabbs.pop();
                            }
                            GeometryType::Triangle => {
                                objects.triangles.pop();
                            }
                        }
                    }
                });
            });

        panel_size
    }

    // returns the size of the debug panel
    fn debug_panel(
        &mut self,
        ctx: &egui::Context,
        fps_counter: &FpsCounter,
        profiler: &RenderProfiler,
        camera: &Camera,
        objects: &Objects,
    ) -> f32 {
        let mut panel_size = 0.0;

        egui::SidePanel::right("debug_panel")
            .resizable(true)
            .default_width(240.0)
            .min_width(80.0)
            .show(ctx, |ui| {
                panel_size = ui.available_width() * ctx.pixels_per_point();

                ui.heading("Frametimes");

                ui.label(format!("FPS: {:.1}", fps_counter.average_fps()));

                ui.label("\nPasses:");
                for (name, time) in profiler.times.iter() {
                    ui.label(format!(
                        "    {}: {:.3} ms",
                        name,
                        time.as_secs_f64() * 1000.0
                    ));
                }

                ui.separator();

                ui.heading("Camera");

                ui.label(format!(
                    "Position: ({:.2}, {:.2}, {:.2})",
                    camera.position.x, camera.position.y, camera.position.z
                ));
                ui.label(format!(
                    "Facing: ({:.2}, {:.2}, {:.2})",
                    camera.forward().x,
                    camera.forward().y,
                    camera.forward().z
                ));
                ui.label(format!("FOV: {:.1}", camera.fov));
                ui.label(format!("Speed: {:.1}", camera.movement_speed));

                ui.separator();

                ui.heading("Objects");

                // Subtract 1 from the length because we have an extra "null" element in each array
                ui.label(format!("Material count: {}", objects.materials.len() - 1));
                ui.label(format!("Sphere count: {}", objects.spheres.len() - 1));
                ui.label(format!("AABB count: {}", objects.aabbs.len() - 1));
                ui.label(format!("Triangle count: {}", objects.triangles.len() - 1));

                ui.separator();

                ui.heading("Controls");

                ui.label("W, A, S, D: move");
                ui.label("Mouse: look");
                ui.label("Arrow up/down: FOV");
                ui.label("Arrow left/right: movement speed");
                ui.label("Escape: show/hide menu");
                ui.label("R: recompile shaders");
            });

        panel_size
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

#[derive(Default, Debug, PartialEq, Eq)]
pub enum GeometryType {
    #[default]
    Sphere,
    Aabb,
    Triangle,
}

#[derive(Default)]
struct ObjectEditor {
    pub material: Material,
    pub geometry_type: GeometryType,
    pub sphere: Sphere,
    pub aabb: Aabb,
    pub triangle: Triangle,
}
