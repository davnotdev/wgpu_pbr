use pollster::FutureExt as _;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

mod vertices;

pub struct App<'window> {
    device: Device,
    queue: Queue,
    surface: Surface<'window>,
    pipeline: RenderPipeline,
    vbo: Buffer,
    ibo: Buffer,
}

impl<'window> App<'window> {
    pub async fn new(window: &'window Window) -> Self {
        let inst = Instance::new(InstanceDescriptor::default());
        let adapter = inst
            .request_adapter(&RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: Features::SPIRV_SHADER_PASSTHROUGH,
                    ..Default::default()
                },
                Default::default(),
            )
            .await
            .unwrap();

        let size = window.inner_size();
        let surface = inst.create_surface(window).unwrap();
        let surface_config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &surface_config);

        let vbo = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(vertices::VERTEX_DATA),
            usage: BufferUsages::VERTEX,
        });

        let ibo = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(vertices::INDEX_DATA),
            usage: BufferUsages::INDEX,
        });

        let vs_raw = include_spirv_raw!("./shaders/vs.spv");
        let fs_raw = include_spirv_raw!("./shaders/fs.spv");

        let vs_shader = unsafe { device.create_shader_module_spirv(&vs_raw) };
        let fs_shader = unsafe { device.create_shader_module_spirv(&fs_raw) };

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &vs_shader,
                entry_point: "main",
                buffers: &[VertexBufferLayout {
                    array_stride: (std::mem::size_of::<f32>() * 3) as u64,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[VertexAttribute {
                        format: VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &fs_shader,
                entry_point: "main",
                targets: &[Some(ColorTargetState {
                    format: surface_config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::all(),
                })],
            }),
            multiview: None,

        });

        Self {
            device,
            queue,
            surface,
            pipeline,
            vbo,
            ibo,
        }
    }

    pub fn render(&mut self) {
        let surface_texture = self.surface.get_current_texture().unwrap();
        let surface_texture_view = surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut command_encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &surface_texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.0,
                            g: 0.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vbo.slice(..));
            render_pass.set_index_buffer(self.ibo.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(0..(vertices::INDEX_DATA.len() as u32), 0, 0..1);
        }
        self.queue.submit([command_encoder.finish()]);
        surface_texture.present();
    }
}

fn main() {
    println!("Hello, world!");
    run().block_on();
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = Window::new(&event_loop).unwrap();
    let main_window_id = window.id();
    let mut app = App::new(&window).await;

    event_loop
        .run(move |ev, elwt| {
            elwt.set_control_flow(ControlFlow::wait_duration(
                std::time::Duration::from_millis(16),
            ));
            match ev {
                Event::WindowEvent { window_id, event } if window_id == main_window_id => {
                    match event {
                        WindowEvent::RedrawRequested => {
                            app.render();
                        }
                        WindowEvent::CloseRequested => {
                            elwt.exit();
                        }
                        _ => {}
                    }
                }
                _ => {}
            };
        })
        .unwrap();
}
