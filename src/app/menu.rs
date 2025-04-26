use std::{
    collections::{LinkedList, VecDeque},
    sync::Arc,
};

use bevy_ecs::{
    event::EventWriter,
    resource::Resource,
    system::{Res, ResMut},
    world::World,
};
use egui::{DragValue, Ui};
use glam::Vec3;
use winit::{keyboard::KeyCode, window::Window};

use crate::{
    ecs::Wrapper,
    egui::EguiRenderState,
    render::{shader::ShaderRecompileEvent, RenderState, WindowResizeEvent},
};

use super::{
    camera::Camera,
    fps::FpsCounter,
    input::Input,
    object::{
        Aabb, AabbPopEvent, AabbPushEvent, Material, MaterialPopEvent, MaterialPushEvent,
        MaterialType, Objects, Sphere, SpherePopEvent, SpherePushEvent, Triangle, TrianglePopEvent,
        TrianglePushEvent,
    },
    renderer::profiler::RenderProfiler,
    time::Time,
};

#[derive(Default, Debug, PartialEq, Eq)]
enum GeometryType {
    #[default]
    Sphere,
    Aabb,
    Triangle,
}

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
    // use linked list because there is almost always only one element, and a Vec would require reallocating the
    // compile error strings
    pub shader_compile_errors: LinkedList<String>,

    pub settings: Settings,
}

impl Menu {
    pub fn init(world: &mut World) {
        let menu = Menu::default();
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
        mut camera: ResMut<Camera>,
        time: Res<Time>,
        objects: Res<Objects>,
        (mut material_push_events, mut material_pop_events): (
            EventWriter<MaterialPushEvent>,
            EventWriter<MaterialPopEvent>,
        ),
        (mut sphere_push_events, mut sphere_pop_events): (
            EventWriter<SpherePushEvent>,
            EventWriter<SpherePopEvent>,
        ),
        (mut aabb_push_events, mut aabb_pop_events): (
            EventWriter<AabbPushEvent>,
            EventWriter<AabbPopEvent>,
        ),
        (mut triangle_push_events, mut triangle_pop_events): (
            EventWriter<TrianglePushEvent>,
            EventWriter<TrianglePopEvent>,
        ),
        mut shader_recompile_events: EventWriter<ShaderRecompileEvent>,
        profiler: Res<RenderProfiler>,
        mut resize_events: EventWriter<WindowResizeEvent>,
    ) {
        let pixels_per_point = egui_render_state.context().pixels_per_point();

        let mut central_panel_offset_from_left = 0.0;
        let mut central_panel_offset_from_right = 0.0;

        // When the user presses escape, disable fullscreen
        if input.keys.just_pressed(KeyCode::Escape) {
            menu.settings.fullscreen = !menu.settings.fullscreen;
        }

        if input.keys.just_pressed(KeyCode::KeyR) {
            shader_recompile_events.send(ShaderRecompileEvent);
        }

        let mut fov_sensitivity = 2.5;
        let mut speed_sensitivity = 1.0;

        if input.keys.pressed(KeyCode::ControlLeft) {
            fov_sensitivity *= 4.0;
            speed_sensitivity *= 4.0;
        }

        if input.keys.pressed(KeyCode::ArrowUp) {
            camera.fov += fov_sensitivity * time.delta().as_secs_f32();
        }

        if input.keys.pressed(KeyCode::ArrowDown) {
            camera.fov -= fov_sensitivity * time.delta().as_secs_f32();
        }

        if input.keys.pressed(KeyCode::ArrowLeft) {
            camera.movement_speed -= speed_sensitivity * time.delta().as_secs_f32();
        }

        if input.keys.pressed(KeyCode::ArrowRight) {
            camera.movement_speed += speed_sensitivity * time.delta().as_secs_f32();
        }

        camera.fov = camera.fov.clamp(30.0, 150.0);
        camera.movement_speed = camera.movement_speed.max(0.0);

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

                    if ui.button("Recompile shaders").clicked() {
                        shader_recompile_events.send(ShaderRecompileEvent);
                    }

                    ui.separator();

                    ui.heading("Path tracing");
                    ui.checkbox(&mut menu.settings.accumulate, "Enable accumulation");
                    ui.checkbox(&mut menu.settings.spectral, "Spectral rendering");

                    ui.separator();

                    ui.heading("Object Editor");

                    ui.collapsing("Material", |ui| {
                        let material = &mut menu.settings.material;

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
                                let material = &mut menu.settings.material;

                                ui.selectable_value(
                                    &mut material.ty,
                                    MaterialType::Lambertian,
                                    "Lambertian",
                                );
                                ui.selectable_value(
                                    &mut material.ty,
                                    MaterialType::Metal,
                                    "Metallic",
                                );
                                ui.selectable_value(
                                    &mut material.ty,
                                    MaterialType::Dielectric,
                                    "Dielectric",
                                );
                            });

                        let show_roughness = matches!(
                            menu.settings.material.ty,
                            MaterialType::Metal | MaterialType::Dielectric
                        );

                        let show_ior =
                            matches!(menu.settings.material.ty, MaterialType::Dielectric);

                        if show_roughness {
                            ui.label("Roughness");
                            ui.add(egui::Slider::new(
                                &mut menu.settings.material.roughness,
                                0.0..=1.0,
                            ));
                        }

                        if show_ior {
                            ui.label("Refractive Index");
                            ui.add(egui::Slider::new(
                                &mut menu.settings.material.ior,
                                0.0..=4.0,
                            ));
                        }
                    });

                    ui.collapsing("Geometry", |ui| {
                        let geo_type = &mut menu.settings.geometry_type;

                        egui::ComboBox::from_label("")
                            .selected_text(format!("{:?}", geo_type))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(geo_type, GeometryType::Sphere, "Sphere");
                                ui.selectable_value(geo_type, GeometryType::Aabb, "Box");
                                ui.selectable_value(geo_type, GeometryType::Triangle, "Triangle");
                            });

                        match geo_type {
                            GeometryType::Sphere => {
                                let sphere = &mut menu.settings.sphere;

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
                                let aabb = &mut menu.settings.aabb;

                                ui.label("Start Bounds");
                                vec3_editor(&mut aabb.min, ui);

                                ui.label("End Bounds");
                                vec3_editor(&mut aabb.max, ui);

                                aabb.max = aabb.max.max(aabb.min + 0.01);
                            }
                            GeometryType::Triangle => {
                                let triangle = &mut menu.settings.triangle;

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

                            if let Some(prev_material) = objects.materials.data.last() {
                                if menu.settings.material == *prev_material {
                                    // To preserve memory, if the material hasn't changed, reuse it with the same index
                                    reuse_material = true;
                                }
                            }

                            let material_index = if reuse_material {
                                objects.materials.data.len() as u32 - 1
                            } else {
                                material_push_events
                                    .send(MaterialPushEvent(menu.settings.material));
                                objects.materials.data.len() as u32
                            };

                            match menu.settings.geometry_type {
                                GeometryType::Sphere => {
                                    menu.settings.sphere.material_index = material_index;
                                    sphere_push_events.send(SpherePushEvent(menu.settings.sphere));
                                }
                                GeometryType::Aabb => {
                                    menu.settings.aabb.material_index = material_index;
                                    aabb_push_events.send(AabbPushEvent(menu.settings.aabb));
                                }
                                GeometryType::Triangle => {
                                    menu.settings.triangle.material_index = material_index;
                                    triangle_push_events
                                        .send(TrianglePushEvent(menu.settings.triangle));
                                }
                            }
                        }

                        if ui.button("Delete Last").clicked() {
                            match menu.settings.geometry_type {
                                GeometryType::Sphere => {
                                    sphere_pop_events.send(SpherePopEvent);
                                }
                                GeometryType::Aabb => {
                                    aabb_pop_events.send(AabbPopEvent);
                                }
                                GeometryType::Triangle => {
                                    triangle_pop_events.send(TrianglePopEvent);
                                }
                            }
                        }
                    });

                    // ui.heading("Scenes");

                    // if ui.button("White furnace").clicked() {}
                });

            egui::SidePanel::right("right")
                .resizable(true)
                .default_width(240.0)
                .min_width(80.0)
                .show(egui_render_state.context(), |ui| {
                    central_panel_offset_from_right = ui.available_width() * pixels_per_point;

                    ui.heading("Frametimes");

                    ui.label(format!("FPS: {:.1}", fps_counter.average_fps()));

                    for (name, time) in profiler.times.iter() {
                        ui.label(format!("{}: {:.3} ms", name, time.as_secs_f64() * 1000.0));
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

                    ui.label(format!("Material count: {}", objects.materials.data.len()));
                    ui.label(format!("Sphere count: {}", objects.spheres.data.len()));
                    ui.label(format!("AABB count: {}", objects.aabbs.data.len()));
                    ui.label(format!("Triangle count: {}", objects.triangles.data.len()));

                    ui.separator();

                    ui.heading("Controls");

                    ui.label("W, A, S, D: move");
                    ui.label("Mouse: look");
                    ui.label("Arrow up/down: FOV");
                    ui.label("Arrow left/right: movement speed");
                    ui.label("Escape: show/hide menu");
                    ui.label("R: recompile shaders");
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
    pub material: Material,
    pub geometry_type: GeometryType,
    pub sphere: Sphere,
    pub aabb: Aabb,
    pub triangle: Triangle,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fullscreen: false,
            accumulate: true,
            spectral: true,
            material: Default::default(),
            geometry_type: Default::default(),
            sphere: Default::default(),
            aabb: Default::default(),
            triangle: Default::default(),
        }
    }
}
