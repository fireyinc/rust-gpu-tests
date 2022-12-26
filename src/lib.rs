#![allow(deprecated)]


use wgpu::util::DeviceExt;
use winit:: {
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder, dpi::LogicalSize,
    window::Window
};



struct State{
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    color: wgpu::Color,
    pipeline: wgpu::RenderPipeline,
    buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3]
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}
 


const VERTICES: &[Vertex] = &[
    Vertex { position: [0., 0.5, 0.], color: [0., 0., 1.] },
    Vertex { position: [-0.5, -0.5, 0.], color: [0., 1., 0.] },
    Vertex { position: [0.5, -0.5, 0.], color: [1., 0., 0.] },
];



impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe {instance.create_surface(window)};
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions{
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface)
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            
            &wgpu::DeviceDescriptor{
                features: wgpu::Features::empty(),

                limits: if cfg!(target_arch = "wasm32"){
                    wgpu::Limits::downlevel_webgl2_defaults()
                }
                else{
                    wgpu::Limits::default()
                },
                label: None
            }, 
            None
        ).await.unwrap();


        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter) [0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto
        };

        surface.configure(&device, &config);

        let color = wgpu::Color {
            r: 0.,
            g: 0.,
            b: 0.,
            a: 0.
        };

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });


        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[]
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState { format: config.format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL})]
            }),
            multiview: None,
        });

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
 


        Self {
            surface,
            device,
            queue,
            config,
            size,
            color,
            pipeline,
            buffer
        }




        
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>){
        if new_size.height > 0 && new_size.width > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool{


        
        match event {


            WindowEvent::CursorMoved { device_id: _, position, modifiers:_} => {

                let offsets = (position.x/self.size.width as f64, position.y/self.size.height as f64);



                self.color = wgpu::Color {
                    r: offsets.0,
                    g: 0.,
                    b: offsets.1,
                    a: 1.
                };
                self.update();

                
                true
            },

            _ => false
        }
    }

    fn update(&mut self){
        
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(self.color),
                                store: true
                        }
                    })
                ],
                depth_stencil_attachment: None


                
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.draw(0..3, 0..1)
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())


    }
}




pub async fn run(height: f64, width: f64) {
    let eloop = EventLoop::new();

    let window = WindowBuilder::new().build(&eloop).expect("crap");
    window.set_title("gpu test");
    window.set_min_inner_size(Some(LogicalSize::new(width, height)));

    let mut state = State::new(&window).await;

    eloop.run(move |e, _, c_flow| {
        *c_flow = ControlFlow::Wait;


        match e {
            


            Event::WindowEvent{ref event, window_id} if window_id == window.id() => if !state.input(event) {
                match event {

                    WindowEvent::CloseRequested => {
                        *c_flow = ControlFlow::Exit;
                    },
                    
                    WindowEvent::Resized(size) => {
                        state.resize(*size);
                    }


                    WindowEvent::ScaleFactorChanged { scale_factor: _, new_inner_size} => {
                        state.resize(**new_inner_size)
                    }

                    

                    _ => ()
                }
            }

            Event::RedrawRequested(window_id) if window_id == window.id() => {
                state.update();
                match state.render() {
                    Ok(_) => (),

                    Err(wgpu::SurfaceError::Lost) => {
                        state.resize(state.size);
                        println!("Swap Chain Lost. Regenerating...");
                    },

                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        *c_flow = ControlFlow::Exit;
                        println!("Out of Memory. Please rerun or check the code for memory issues.");
                    },

                    Err(e) => println!("{}", e),

                }


            },

            Event::MainEventsCleared => {
                window.request_redraw();
            },


            _ => ()
        }
    });




    
}