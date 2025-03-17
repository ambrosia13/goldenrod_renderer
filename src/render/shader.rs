use std::{
    borrow::Cow,
    collections::HashSet,
    path::{Path, PathBuf},
};

use bevy_ecs::{
    component::Component,
    entity::Entity,
    event::{Event, EventReader, EventWriter},
    system::{Query, ResMut},
};
use regex::Regex;

use crate::app::menu::Menu;

use super::{GpuHandle, RenderResourceUpdateEvent};

fn path_name_to_string<P: AsRef<Path>>(path: P) -> String {
    // ew
    path.as_ref()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
}

fn resolve_includes(mut source: String, parent_dir: &Path) -> Result<String, std::io::Error> {
    let mut included = HashSet::new();

    let regex = Regex::new(r#"#include ([\w/\.]+)"#).unwrap();

    while let Some(regex_match) = regex.find(&source) {
        let include_arg = regex_match
            .as_str()
            .split_ascii_whitespace()
            .nth(1)
            .unwrap();

        let relative_path = Path::new(include_arg);
        let include_path = parent_dir.join(relative_path);

        if !included.contains(&include_path) {
            let include_source = std::fs::read_to_string(&include_path)?;

            source = regex.replace(&source, &include_source).to_string();
            included.insert(include_path);
        } else {
            source = regex.replace(&source, "").to_string();
        }
    }

    Ok(source)
}

#[derive(Clone, Copy, Debug)]
pub enum ShaderBackend {
    Wgsl,
    Spirv,
}

// impl ShaderBackend {
//     pub fn guess<P: AsRef<Path>>(path: P) -> Self {
//         let path = path.as_ref();

//         path.extension()
//     }
// }

pub enum ShaderModuleDescriptor<'a> {
    Default(wgpu::ShaderModuleDescriptor<'a>),
    Spirv(wgpu::ShaderModuleDescriptorSpirV<'a>),
}

pub struct ShaderMetadata {
    pub name: String,
    pub path: PathBuf,
    pub backend: ShaderBackend,
}

pub struct ShaderSource {
    metadata: ShaderMetadata,
    source: Option<Vec<u8>>,
}

impl ShaderSource {
    fn load<P: AsRef<Path>>(path: P, backend: ShaderBackend) -> Self {
        let name = path_name_to_string(&path);
        let path = path.as_ref().to_owned();

        let metadata = ShaderMetadata {
            name,
            path,
            backend,
        };

        fn read_shader_source<U: AsRef<Path>>(
            path: U,
            backend: ShaderBackend,
        ) -> std::io::Result<Vec<u8>> {
            let parent_path = std::env::current_dir()?;
            let path = parent_path.join(path);

            match backend {
                ShaderBackend::Wgsl => {
                    let source = std::fs::read_to_string(&path)?;
                    let source = resolve_includes(source, &parent_path)?;

                    Ok(source.into_bytes())
                }
                ShaderBackend::Spirv => Ok(std::fs::read(&path)?),
            }
        }

        let source = read_shader_source(&metadata.path, backend).ok();

        Self { metadata, source }
    }

    pub fn load_wgsl<P: AsRef<Path>>(path: P) -> Self {
        Self::load(path, ShaderBackend::Wgsl)
    }

    pub fn load_spirv<P: AsRef<Path>>(path: P) -> Self {
        Self::load(path, ShaderBackend::Spirv)
    }

    pub fn reload(&mut self) {
        let path = &self.metadata.path;
        *self = Self::load(path, self.metadata.backend);
    }

    pub fn is_fallback(&self) -> bool {
        self.source.is_none()
    }

    pub fn make_fallback(&mut self) {
        self.source = None;
    }

    pub fn backend(&self) -> ShaderBackend {
        self.metadata.backend
    }

    fn source_str(&self) -> Option<&str> {
        match self.backend() {
            ShaderBackend::Wgsl => Some(std::str::from_utf8(self.source.as_ref()?).unwrap()),
            ShaderBackend::Spirv => panic!("Can't get source strings for binary Spir-V format"),
        }
    }

    fn source_words(&self) -> Option<Cow<'_, [u32]>> {
        match self.backend() {
            ShaderBackend::Wgsl => panic!("Can't get source words for wgsl"),
            ShaderBackend::Spirv => Some(wgpu::util::make_spirv_raw(self.source.as_ref()?)),
        }
    }

    pub fn descriptor(&self) -> ShaderModuleDescriptor {
        match self.is_fallback() {
            false => match self.backend() {
                ShaderBackend::Wgsl => {
                    let source_str = self.source_str();

                    ShaderModuleDescriptor::Default(wgpu::ShaderModuleDescriptor {
                        label: Some(&self.metadata.name),
                        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source_str.unwrap())),
                    })
                }
                ShaderBackend::Spirv => {
                    let source_words = self.source_words();

                    ShaderModuleDescriptor::Spirv(wgpu::ShaderModuleDescriptorSpirV {
                        label: Some(&self.metadata.name),
                        source: source_words.unwrap(),
                    })
                }
            },
            true => ShaderModuleDescriptor::Default(self.fallback_descriptor()),
        }
    }

    pub fn fallback_descriptor(&self) -> wgpu::ShaderModuleDescriptor<'_> {
        wgpu::ShaderModuleDescriptor {
            label: Some(&self.metadata.name),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("assets/fallback.wgsl"))),
        }
    }
}

#[derive(Component)]
pub struct Shader {
    source: ShaderSource,
    module: wgpu::ShaderModule,

    gpu_handle: GpuHandle,
}

impl Shader {
    pub fn new(gpu_handle: impl Into<GpuHandle>, mut source: ShaderSource) -> Self {
        let gpu_handle: GpuHandle = gpu_handle.into();

        match source.descriptor() {
            ShaderModuleDescriptor::Default(shader_module_descriptor) => {
                gpu_handle
                    .device
                    .push_error_scope(wgpu::ErrorFilter::Validation);

                let mut module = gpu_handle
                    .device
                    .create_shader_module(shader_module_descriptor);

                let compile_error = pollster::block_on(gpu_handle.device.pop_error_scope());

                if compile_error.is_some() {
                    source.make_fallback();
                    module = gpu_handle
                        .device
                        .create_shader_module(source.fallback_descriptor());
                }

                Self {
                    source,
                    module,
                    gpu_handle,
                }
            }
            ShaderModuleDescriptor::Spirv(shader_module_descriptor_spir_v) => {
                let module = unsafe {
                    gpu_handle
                        .device
                        .create_shader_module_spirv(&shader_module_descriptor_spir_v)
                };

                Self {
                    source,
                    module,
                    gpu_handle,
                }
            }
        }
    }

    pub fn source(&self) -> &ShaderSource {
        &self.source
    }

    pub fn module(&self) -> &wgpu::ShaderModule {
        &self.module
    }

    pub fn recreate(&mut self) -> Option<wgpu::Error> {
        self.source.reload();

        let mut error = None;

        self.module = match self.source.descriptor() {
            ShaderModuleDescriptor::Default(shader_module_descriptor) => {
                self.gpu_handle
                    .device
                    .push_error_scope(wgpu::ErrorFilter::Validation);

                let mut module = self
                    .gpu_handle
                    .device
                    .create_shader_module(shader_module_descriptor);

                let compile_error = pollster::block_on(self.gpu_handle.device.pop_error_scope());

                if let Some(compile_error) = compile_error {
                    error = Some(compile_error);

                    self.source.make_fallback();
                    module = self
                        .gpu_handle
                        .device
                        .create_shader_module(self.source.fallback_descriptor());
                }

                module
            }
            ShaderModuleDescriptor::Spirv(shader_module_descriptor_spir_v) => unsafe {
                self.gpu_handle
                    .device
                    .create_shader_module_spirv(&shader_module_descriptor_spir_v)
            },
        };

        error
    }
}

#[derive(Event)]
pub struct ShaderRecompileEvent;

pub fn recompile_shaders(
    mut shader_query: Query<(Entity, &mut Shader)>,
    mut recompile_events: EventReader<ShaderRecompileEvent>,
    mut resource_update_events: EventWriter<RenderResourceUpdateEvent>,
    mut menu: ResMut<Menu>,
) {
    // If there's no recompile events we don't need to do anything
    if recompile_events.read().count() == 0 {
        return;
    }

    for (entity, mut shader) in shader_query.iter_mut() {
        shader.source.reload();
        let error = shader.recreate();

        // If there is an error, send it to menu so it can be displayed
        menu.shader_compile_error = error.map(|e| e.to_string());

        // mark this shader as updated so things that depend on it (e.g. pipelines) can update accordingly
        resource_update_events.send(RenderResourceUpdateEvent(entity));
    }
}
