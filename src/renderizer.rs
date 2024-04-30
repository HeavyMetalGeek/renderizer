use crate::vertex::{MeshVertex, Vertex, TRIANGLE};
use eframe::{
    egui_wgpu::wgpu::util::DeviceExt,
    egui_wgpu::{self, wgpu},
};

pub struct Renderizer;

impl Renderizer {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>) -> Self {
        let wgpu_render_state = cc.wgpu_render_state.as_ref().unwrap();
        let device = &wgpu_render_state.device;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Basic Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("basic_shader.wgsl").into()),
        });
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(TRIANGLE),
            usage: wgpu::BufferUsages::VERTEX,
        });
        //let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //    label: Some("Bind Group Layout"),
        //    entries: &[wgpu::BindGroupLayoutEntry {
        //        binding: 0,
        //        visibility: wgpu::ShaderStages::VERTEX,
        //        ty: wgpu::BindingType::Buffer {
        //            ty: wgpu::BufferBindingType::Uniform,
        //            has_dynamic_offset: false,
        //            min_binding_size: None,
        //        },
        //        count: None,
        //    }],
        //});
        //let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //    label: Some("Bind Group"),
        //    layout: &bind_group_layout,
        //    entries: &[wgpu::BindGroupEntry {
        //        binding: 0,
        //        resource: vertex_buffer.as_entire_binding(),
        //    }],
        //});
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            //bind_group_layouts: &[&bind_group_layout],
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[MeshVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu_render_state.target_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        wgpu_render_state
            .renderer
            .write()
            .callback_resources
            .insert(RenderizerResources {
                pipeline,
                vertex_buffer,
                //bind_group,
            });
        Self
    }

    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, _res) = ui.allocate_exact_size(egui::Vec2::splat(300.0), egui::Sense::drag());
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            RenderizerCallback,
        ));
    }
}

impl eframe::App for Renderizer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    self.custom_painting(ui);
                });
            });
        });
    }
}

struct RenderizerCallback;

impl egui_wgpu::CallbackTrait for RenderizerCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &RenderizerResources = resources.get().unwrap();
        resources.prepare(device, queue);
        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        resources: &'a egui_wgpu::CallbackResources,
    ) {
        let resources: &RenderizerResources = resources.get().unwrap();
        resources.paint(render_pass);
    }
}

struct RenderizerResources {
    pipeline: wgpu::RenderPipeline,
    //bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
}

impl RenderizerResources {
    fn prepare(&self, _device: &wgpu::Device, _queue: &wgpu::Queue) {
        //
    }

    fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        //render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
}
