#![allow(dead_code)]

mod shader_importer;

use wgpu::util::{DeviceExt};
use winit::{
    event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};
use std::sync::mpsc::channel;

const SIZEE: u64 = 1080*1920;
struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    render_pipeline: Option<wgpu::RenderPipeline>,
    compute_pipeline: Option<wgpu::ComputePipeline>,
    work_group_count: u32,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    
    importer: shader_importer::Importer,
    compile_status: bool,
    shader_code: Option<String>,
    
    stuff: Stuff,
    compute_buffer: wgpu::Buffer, // general buffer to play with
    stuff_buffer: wgpu::Buffer,
    bind_group_layouts: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    time: std::time::Instant,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None, // Trace path
        ).await.unwrap();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
        };
        surface.configure(&device, &config);

        let compute_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Compute Buffer")),
            contents: bytemuck::cast_slice(&vec![0u32 ; SIZEE as usize]),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let stuff = Stuff::new();
        let stuff_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("stuff buffer"),
                contents: bytemuck::cast_slice(&[stuff]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bind_group_layouts = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bind group layouts"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,//wgpu::BufferSize::new(SIZEE*4),
                    },
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("stuff bind group"),
            layout: &bind_group_layouts,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: stuff_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: compute_buffer.as_entire_binding(),
                },
            ],
        });

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX
            }
        );
        let num_vertices = VERTICES.len() as u32;

        let mut state = Self { 
            surface, device, queue, config, size, render_pipeline: None, compute_pipeline: None, work_group_count: 200,
            vertex_buffer, num_vertices, 
            stuff, stuff_buffer, compute_buffer, bind_group_layouts, bind_group,
            importer: shader_importer::Importer::new("./src/shader.wgsl"),
            compile_status: false,
            shader_code: None,
            time: std::time::Instant::now(),
        };
        state.compile();
        state
    }

    fn fallback_shader() -> String {
        String::from("
            [[stage(vertex)]]
            fn main() -> [[builtin(position)]] vec4<f32> {
                return vec4<f32>(1.0);
            }
            [[stage(fragment)]]
            fn main([[builtin(position)]] pos: vec4<f32>) -> [[location(0)]] vec4<f32> {
                return vec4<f32>(1.0);
            }
            [[stage(compute), workgroup_size(64)]]
            fn main([[builtin(global_invocation_id)]] global_invocation_id: vec3<u32>) {
            }
        ")
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.stuff.width = self.size.width as f32;
            self.stuff.height = self.size.height as f32;    
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.stuff.cursor_x = position.x as f32;
                self.stuff.cursor_y = position.y as f32;
            },
            WindowEvent::MouseInput {button, state, ..} => {
                let p_or_r = match state {
                    winit::event::ElementState::Pressed => 1,
                    winit::event::ElementState::Released => 0,
                };
                match button {
                    winit::event::MouseButton::Left => self.stuff.mouse_left = p_or_r,
                    winit::event::MouseButton::Right => self.stuff.mouse_right = p_or_r,
                    winit::event::MouseButton::Middle => self.stuff.mouse_middle = p_or_r,
                    _ => (),
                }
            },
            WindowEvent::MouseWheel {delta, ..} => {
                match delta {
                    // winit::event::MouseScrollDelta::PixelDelta(pp) => self.stuff.scroll += (pp.y+pp.x) as f32,
                    winit::event::MouseScrollDelta::LineDelta(x, y) => self.stuff.scroll += (x+y) as f32,
                    _ => (),
                }
            },
            _ => return false,
        };
        true
    }

    fn update(&mut self) {
        self.stuff.time += self.time.elapsed().as_secs_f32();

        self.queue.write_buffer(&self.stuff_buffer, 0, bytemuck::cast_slice(&[self.stuff]));
        self.compile();
    }

    fn compile(&mut self) {
        let shader_code = {
            if self.shader_code.is_none() {
                Some(Self::fallback_shader())
            } else if self.compile_status {
                self.importer.check_and_import()
            } else {
                self.importer.import()
            }
        };
        if shader_code.is_none() {return}
        if !self.compile_status && self.shader_code == shader_code {return}
        self.shader_code = shader_code;

        self.compile_render_shaders();
        self.compile_compute_shaders();
    }

    fn compile_render_shaders(&mut self) {
        let (tx, rx) = channel::<wgpu::Error>();
        self.device.on_uncaptured_error(move |e: wgpu::Error| {
            tx.send(e).expect("sending error failed");
        });
        let shader = self.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(self.shader_code.as_ref().unwrap())),
        });
        let render_pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &self.bind_group_layouts,
            ],
            push_constant_ranges: &[],
        });
        let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "main", // 1.
                buffers: &[Vertex::desc()], // 2.
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState { // 4.
                    format: self.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLAMPING
                clamp_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
        });

        if let Ok(err) = rx.try_recv() {
            self.compile_status = false;
            println!("{}", err);
            return;
        }
        dbg!("render shaders compiled");
        self.compile_status = true;
        self.render_pipeline = Some(render_pipeline);
    }

    fn compile_compute_shaders(&mut self) {
        let (tx, rx) = channel::<wgpu::Error>();
        self.device.on_uncaptured_error(move |e: wgpu::Error| {
            tx.send(e).expect("sending error failed");
        });
        let shader = self.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(self.shader_code.as_ref().unwrap())),
        });
        let compute_pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[
                &self.bind_group_layouts,
            ],
            push_constant_ranges: &[],
        });
        let compute_pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        if let Ok(err) = rx.try_recv() {
            self.compile_status = false;
            println!("{}", err);
            return;
        }
        dbg!("compute shaders compiled");
        self.compile_status = true;
        self.compute_pipeline = Some(compute_pipeline);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            compute_pass.set_pipeline(self.compute_pipeline.as_ref().unwrap());
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch(self.work_group_count, 1, 1);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(self.render_pipeline.as_ref().unwrap());
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.draw(0..self.num_vertices, 0..1);
        }
    
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    
        Ok(())
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = pollster::block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    },
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so we have to dereference it twice
                        state.resize(**new_inner_size);
                    },
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ]
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [1.0, 1.0, 0.0] },
    Vertex { position: [-1.0, 1.0, 0.0] },
    Vertex { position: [1.0, -1.0, 0.0] },

    Vertex { position: [1.0, -1.0, 0.0] },
    Vertex { position: [-1.0, 1.0, 0.0] },
    Vertex { position: [-1.0, -1.0, 0.0] },
];

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]

struct Stuff {
    width: f32,
    height: f32,
    time: f32,
    cursor_x: f32,
    cursor_y: f32,
    scroll: f32,

    // TODO: figure out how to send bool or compress this into a single variable
      // can shove inside a u32 and do (variable & u32(<2^n>)) to get it out
    mouse_left: u32,
    mouse_right: u32,
    mouse_middle: u32,
}

impl Stuff {
    fn new() -> Self {
        Self {
            width: 100.0,
            height: 100.0,
            time: 0.0,
            cursor_x: 0.0,
            cursor_y: 0.0,

            mouse_left: 0,
            mouse_right: 0,
            mouse_middle: 0,
            scroll: 0.0,
        }
    }
}