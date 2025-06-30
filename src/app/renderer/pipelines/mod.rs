use wgputil::{shader::ShaderSource, GpuHandle};

pub mod display_pipelines;
pub mod material_pipelines;

type RenderPipelineDescriptorSupplierSeparateModules<'a> = Box<
    dyn FnMut(&wgpu::ShaderModule, &wgpu::ShaderModule) -> wgpu::RenderPipelineDescriptor<'a>
        + Send
        + Sync,
>;

type RenderPipelineDescriptorSupplierSingleModule<'a> =
    Box<dyn FnMut(&wgpu::ShaderModule) -> wgpu::RenderPipelineDescriptor<'a> + Send + Sync>;

type ComputePipelineDescriptorSupplier<'a> =
    Box<dyn FnMut(&wgpu::ShaderModule) -> wgpu::ComputePipelineDescriptor<'a> + Send + Sync>;

enum RenderPipelineMetadata<'a> {
    SeparateModules {
        vertex_source: ShaderSource,
        vertex_module: wgpu::ShaderModule,

        fragment_source: ShaderSource,
        fragment_module: wgpu::ShaderModule,

        desc_supplier: RenderPipelineDescriptorSupplierSeparateModules<'a>,
    },
    SingleModule {
        source: ShaderSource,
        module: wgpu::ShaderModule,

        desc_supplier: RenderPipelineDescriptorSupplierSingleModule<'a>,
    },
}

struct ComputePipelineMetadata<'a> {
    source: ShaderSource,
    module: wgpu::ShaderModule,

    desc_supplier: ComputePipelineDescriptorSupplier<'a>,
}

pub struct ManagedRenderPipeline<'a> {
    metadata: RenderPipelineMetadata<'a>,
    pipeline: wgpu::RenderPipeline,
}

impl<'a> ManagedRenderPipeline<'a> {
    pub fn with_separate_modules<F>(
        gpu: &GpuHandle,
        mut vertex_source: ShaderSource,
        mut fragment_source: ShaderSource,
        mut desc_supplier: F,
    ) -> Self
    where
        F: FnMut(&wgpu::ShaderModule, &wgpu::ShaderModule) -> wgpu::RenderPipelineDescriptor<'a>
            + 'static
            + Send
            + Sync,
    {
        let (vertex_module, _error) =
            wgputil::shader::create_or_fallback(&gpu.device, &mut vertex_source);

        let (fragment_module, _error) =
            wgputil::shader::create_or_fallback(&gpu.device, &mut fragment_source);

        let desc = desc_supplier(&vertex_module, &fragment_module);
        let desc_supplier = Box::new(desc_supplier);

        let metadata = RenderPipelineMetadata::SeparateModules {
            vertex_source,
            vertex_module,
            fragment_source,
            fragment_module,
            desc_supplier,
        };

        let pipeline = gpu.device.create_render_pipeline(&desc);

        Self { metadata, pipeline }
    }

    pub fn with_single_module<F>(
        gpu: &GpuHandle,
        mut source: ShaderSource,
        mut desc_supplier: F,
    ) -> Self
    where
        F: FnMut(&wgpu::ShaderModule) -> wgpu::RenderPipelineDescriptor<'a> + 'static + Send + Sync,
    {
        let (module, _error) = wgputil::shader::create_or_fallback(&gpu.device, &mut source);

        let desc = desc_supplier(&module);
        let desc_supplier = Box::new(desc_supplier);

        let metadata = RenderPipelineMetadata::SingleModule {
            source,
            module,
            desc_supplier,
        };

        let pipeline = gpu.device.create_render_pipeline(&desc);

        Self { metadata, pipeline }
    }

    pub fn reload(&mut self, gpu: &GpuHandle) {
        let desc = match &mut self.metadata {
            RenderPipelineMetadata::SeparateModules {
                vertex_source,
                vertex_module,
                fragment_source,
                fragment_module,
                desc_supplier,
            } => {
                vertex_source.reload();
                fragment_source.reload();

                let (module, _error) =
                    wgputil::shader::create_or_fallback(&gpu.device, vertex_source);
                *vertex_module = module;

                let (module, _error) =
                    wgputil::shader::create_or_fallback(&gpu.device, fragment_source);
                *fragment_module = module;

                desc_supplier(vertex_module, fragment_module)
            }
            RenderPipelineMetadata::SingleModule {
                source,
                module,
                desc_supplier,
            } => {
                source.reload();

                let (new_module, _error) = wgputil::shader::create_or_fallback(&gpu.device, source);
                *module = new_module;

                desc_supplier(module)
            }
        };

        self.pipeline = gpu.device.create_render_pipeline(&desc);
    }

    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}

pub struct ManagedComputePipeline<'a> {
    metadata: ComputePipelineMetadata<'a>,
    pipeline: wgpu::ComputePipeline,
}

impl<'a> ManagedComputePipeline<'a> {
    pub fn new<F>(gpu: &GpuHandle, mut source: ShaderSource, mut desc_supplier: F) -> Self
    where
        F: FnMut(&wgpu::ShaderModule) -> wgpu::ComputePipelineDescriptor<'a>
            + 'static
            + Send
            + Sync,
    {
        let (module, _error) = wgputil::shader::create_or_fallback(&gpu.device, &mut source);

        let desc = desc_supplier(&module);
        let desc_supplier = Box::new(desc_supplier);

        let metadata = ComputePipelineMetadata {
            source,
            module,
            desc_supplier,
        };

        let pipeline = gpu.device.create_compute_pipeline(&desc);

        Self { metadata, pipeline }
    }

    pub fn reload(&mut self, gpu: &GpuHandle) {
        self.metadata.source.reload();

        let (new_module, _error) =
            wgputil::shader::create_or_fallback(&gpu.device, &mut self.metadata.source);
        self.metadata.module = new_module;

        let desc = (self.metadata.desc_supplier)(&self.metadata.module);
        self.pipeline = gpu.device.create_compute_pipeline(&desc);
    }

    pub fn pipeline(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }
}
