use crate::vertex::{MeshVertex, Vertex, TRIANGLE};
use eframe::{
    egui_wgpu::{
        self,
        wgpu::{self, util::DeviceExt},
    },
    wgpu::util::RenderEncoder,
};
use rand::prelude::*;

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
        let instances = vec![
            Instance {
                pos: glam::Vec3::new(-0.5, -0.5, 0.0),
                rot: glam::Quat::IDENTITY,
            },
            Instance {
                pos: glam::Vec3::new(0.5, -0.5, 0.0),
                rot: glam::Quat::IDENTITY,
            },
            Instance {
                pos: glam::Vec3::new(0.0, 0.5, 0.0),
                rot: glam::Quat::IDENTITY,
            },
            Instance {
                pos: glam::Vec3::new(0.5, 0.5, 0.0),
                rot: glam::Quat::IDENTITY,
            },
        ];
        let instance_data: Vec<InstanceRaw> = instances.iter().map(Instance::to_raw).collect();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
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
                buffers: &[MeshVertex::desc(), InstanceRaw::desc()],
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
                instances,
                instance_buffer,
            });
        Self
    }

    fn custom_painting(&mut self, ui: &mut egui::Ui, add_instance: bool) {
        let (rect, _res) =
            ui.allocate_exact_size(egui::Vec2::new(1920.0, 1080.0), egui::Sense::drag());
        let instance = if add_instance {
            let mut rng = rand::thread_rng();
            let x = rng.gen::<f32>() - 0.5;
            let y = rng.gen::<f32>() - 0.5;
            //let z = rng.gen::<f32>() - 0.5;
            Some(Instance {
                pos: glam::Vec3::new(x, y, 0.0),
                rot: glam::Quat::IDENTITY,
            })
        } else {
            None
        };
        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            RenderizerCallback {
                new_instance: instance,
            },
        ));
    }
}

impl eframe::App for Renderizer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let add_instance = ui.button("Add Instance").clicked();
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    self.custom_painting(ui, add_instance);
                });
            });
        });
    }
}

struct RenderizerCallback {
    new_instance: Option<Instance>,
}

impl egui_wgpu::CallbackTrait for RenderizerCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &mut RenderizerResources = resources.get_mut().unwrap();
        match &self.new_instance {
            Some(i) => {
                resources.instances.push(i.clone());
                let instance_data: Vec<InstanceRaw> =
                    resources.instances.iter().map(Instance::to_raw).collect();
                resources.instance_buffer =
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Instance Buffer"),
                        contents: bytemuck::cast_slice(&instance_data),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    });
                resources.prepare(device, queue, Some(bytemuck::cast_slice(&[i.to_raw()])));
            }
            None => resources.prepare(device, queue, None),
        }
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
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
}

impl RenderizerResources {
    fn prepare(&self, _device: &wgpu::Device, queue: &wgpu::Queue, instance: Option<&[u8]>) {
        //if let Some(i) = instance {
        //    queue.write_buffer(&self.instance_buffer, 0, i);
        //}
    }

    fn paint<'rp>(&'rp self, render_pass: &mut wgpu::RenderPass<'rp>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.draw(0..3, 0..self.instances.len() as _)
    }
}

#[derive(Clone, Debug)]
struct Instance {
    pos: glam::Vec3,
    rot: glam::Quat,
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        let mat = glam::Mat4::from_rotation_translation(self.rot, self.pos);
        InstanceRaw {
            x_axis: mat.x_axis.into(),
            y_axis: mat.y_axis.into(),
            z_axis: mat.z_axis.into(),
            w_axis: mat.w_axis.into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    x_axis: [f32; 4],
    y_axis: [f32; 4],
    z_axis: [f32; 4],
    w_axis: [f32; 4],
}

impl InstanceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
