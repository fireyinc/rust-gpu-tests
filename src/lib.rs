#![allow(deprecated)]


use image::GenericImageView;
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
    v_buffer: wgpu::Buffer,
    i_buffer: wgpu::Buffer,
    num_vertices: u32,
    num_indices: u32,
    bind_group: wgpu::BindGroup,

}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    tex_coord: [f32; 2]
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode:  wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        }
    }
}


unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}
 


const VERTICES: &[Vertex] = &[
    Vertex { position: [0., 0.5, 0.], tex_coord: [0., 0.5] },
    Vertex { position: [-0.25, -0.5, 0.], tex_coord: [-0.25, -0.5] },
    Vertex { position: [0.25, -0.5, 0.], tex_coord: [0.25, -0.5] },

    
    Vertex { position: [0.4, 0.1, 0.], tex_coord: [0.4, 0.1] },
    Vertex { position: [-0.4, 0.1, 0.], tex_coord: [-0.4, 0.1] },
        
];


const INDICES: &[u16] = &[
    0, 1, 2,
    3, 0, 2,
    4, 1, 0
];



impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe {instance.create_surface(window)};
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions{
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface)
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            
            &wgpu::DeviceDescriptor{
                features: wgpu::Features::POLYGON_MODE_LINE,

                limits: if cfg!(target_arch = "wasm32"){
                    wgpu::Limits::downlevel_webgl2_defaults()
                }
                else{
                    wgpu::Limits::default()
                },
                label: None
            }, 
            None
        ).await.expect("fak");

        let diffuse_bytes = include_bytes!("ad_dc10_tex.png");
        let diffuse_img = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_img.as_rgba8().unwrap();
        
        let tex_dimensions = diffuse_img.dimensions();

        use image::GenericImageView;


        let tex_size = wgpu::Extent3d {
            width: tex_dimensions.0,
            height: tex_dimensions.1,
            depth_or_array_layers: 1,
        };

        let diffuse_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size: tex_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });


        queue.write_texture( 
            wgpu::ImageCopyTexture {
                texture: &diffuse_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            }, 
            
            &diffuse_rgba,

            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * tex_dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(tex_dimensions.1),
            }, 

            tex_size
        );

        let diff_tex_view = diffuse_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let diff_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let tex_bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                }

            ],
        });
        

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &tex_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diff_tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diff_sampler),
                },
            ],
        });
        

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
                bind_group_layouts: &[
                    &tex_bind_layout
                ],
                push_constant_ranges: &[],
            });


        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::desc()
                ]
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

      
        let v_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let i_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });


        let num_vertices = VERTICES.len() as u32;
        let num_indices = INDICES.len() as u32;

        Self {
            surface,
            device,
            queue,
            config,
            size,
            color,
            pipeline,
            v_buffer,
            i_buffer,
            num_vertices,
            num_indices,
            bind_group,
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


            // WindowEvent::CursorMoved { device_id: _, position, modifiers:_} => {

            //     let offsets = (position.x/self.size.width as f64, position.y/self.size.height as f64);



            //     self.color = wgpu::Color {
            //         r: offsets.0,
            //         g: 0.,
            //         b: offsets.1,
            //         a: 1.
            //     };
            //     self.update();

                
            //     true
            // },

                

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
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.v_buffer.slice(..));
            render_pass.set_index_buffer(self.i_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw(0..self.num_vertices, 0..1);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

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