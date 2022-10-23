use std::{iter, mem};
use bytemuck::{Pod, Zeroable};
use wgpu::{util::DeviceExt, TextureUsages, Sampler};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopBuilder},
    window::{Window, WindowBuilder},
    platform::web
};

use wasm_bindgen_futures::spawn_local;


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2]
}

const VERTICES: &[Vertex] = &[
    Vertex {
        // a
        position: [-1.0, -1.0], //[-0.5, -0.5],
        tex_coords: [0.0, 1.0]
    },
    Vertex {
        // b
        position: [1.0, -1.0], //[0.5, -0.5],
        tex_coords: [1.0, 1.0]
    },
    Vertex {
        // d
        position: [-1.0, 1.0], //[-0.5, 0.5],
        tex_coords: [0.0, 0.0]
    },
    Vertex {
        // d
        position: [-1.0, 1.0], //[-0.5, 0.5],
        tex_coords: [0.0, 0.0]
    },
    Vertex {
        // b
        position: [1.0, -1.0], //[0.5, -0.5],
        tex_coords: [1.0, 1.0]
    },
    Vertex {
        // c
        position: [1.0, 1.0], //[0.5, 0.5],
        tex_coords: [1.0, 0.0]
    },
];

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] = 
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InputUniform {
    effect: i32,
    fill_mode: i32,
    window_ratio: f32,
    img_ratio: f32,
}

impl InputUniform {
    fn new(window_ratio: f32) -> Self {
        Self {
            effect: 0,
            fill_mode: 0,
            window_ratio: window_ratio,
            img_ratio: 0f32
        }
    }

    fn step(&mut self) {
        if self.effect < 6 {
            self.effect += 1;
        } else {
            self.effect = 0;
        }
    }

    fn toggle_fill(&mut self) {
        if self.fill_mode == 0 {
            self.fill_mode = 1;
        } else {
            self.fill_mode = 0;
        }
    }
    
}

pub struct WebImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>
}


// use image::ImageBuffer;
// use image::Rgba;
struct ImageUniform {
    updated: bool,
    img: Option<WebImage>,
    diffuse_texture: Texture,
    texture_bind_group_layout: BindGroupLayout,
    diffuse_texture_view: TextureView,
    diffuse_sampler: Sampler,
}
use wgpu::{TextureView, Texture, BindGroupLayout };

impl ImageUniform {
    fn new(diffuse_texture: Texture, 
        texture_bind_group_layout: BindGroupLayout,
        diffuse_texture_view: TextureView,
        diffuse_sampler: Sampler
        ) -> Self {
        ImageUniform { 
            updated: false, 
            img: None, 
            diffuse_texture: diffuse_texture,
            texture_bind_group_layout: texture_bind_group_layout,
            diffuse_texture_view,
            diffuse_sampler,
        }
    }

    fn img_copy_tex(&self) -> wgpu::ImageCopyTexture {
        wgpu::ImageCopyTexture {
            texture: &self.diffuse_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        }
    }

    fn update_tex(&mut self, img: WebImage) {
        self.img = Some(img);
        self.updated = true;
    }

    fn get_pixels(&self) -> &Vec<u8> {
        match &self.img {
            Some(img) => &img.data,
            None => panic!("Requested pixels of empty image") 
        }
    }

    fn get_dims(&self) -> (u32, u32) {
        match &self.img {
            Some(img) => (img.width, img.height),
            None => (1 as u32, 1 as u32)
        }
    }

    fn get_texture_size(&self) -> wgpu::Extent3d {
        let dimensions = self.get_dims();
        wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        }
    }

    fn image_data_layout(&self) -> wgpu::ImageDataLayout {
        let dimensions = self.get_dims();
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
            rows_per_image: std::num::NonZeroU32::new(dimensions.1),
        }
    }
    fn create_view(&self) -> TextureView {
        self.diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    diffuse_bind_group: wgpu::BindGroup,
    image_tex_uniform: ImageUniform,
    input_uniform: InputUniform,
    input_buffer: wgpu::Buffer,
    input_bind_group: wgpu::BindGroup,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size= window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false
            })
            .await
            .unwrap();
        
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_webgl2_defaults(),
                }, 
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo
        };
        surface.configure(&device, &config);

        let diffuse_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: 500 as u32,
                    height: 500 as u32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1, 
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // Most images are stored using sRGB so we need to reflect that here.
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("diffuse_texture"),
            }
        );

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[0 as u8; 1_000_000 as usize],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * 500),
                rows_per_image: std::num::NonZeroU32::new(500),
            },
            wgpu::Extent3d {
                width: 500 as u32,
                height: 500 as u32,
                depth_or_array_layers: 1,
            },
        );
        // We don't need to configure the texture view much, so let's
        // let wgpu define it.
        let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::MirrorRepeat,
            address_mode_v: wgpu::AddressMode::MirrorRepeat,
            address_mode_w: wgpu::AddressMode::MirrorRepeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        
        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                    },
                ],
                label: Some("diffuse_bind_group"),
            }
        );


        let mut input_uniform = InputUniform::new((size.width as f32) / (size.height as f32));
        let input_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("exposure toggle"),
                contents: bytemuck::cast_slice(&[input_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let input_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<[i32; 4]>() as u64),
                    },
                    count: None,
                }
            ],
            label: Some("exposure_toggle_bind_group_layout"),
        });

        let input_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &input_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_buffer.as_entire_binding(),
                }
            ],
            label: Some("input_bind_group"),
        });

            


        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("effect-shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &input_bind_group_layout,
                ],
            push_constant_ranges: &[]
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent{ 
                            src_factor: wgpu::BlendFactor::SrcAlpha, 
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha, 
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX
        });

        let mut image_tex_uniform = ImageUniform::new(diffuse_texture, texture_bind_group_layout, diffuse_texture_view, diffuse_sampler);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            pipeline,
            vertex_buffer,
            diffuse_bind_group,
            image_tex_uniform: image_tex_uniform,
            input_uniform,
            input_buffer,
            input_bind_group,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        false // no input events to capture
    }

    fn update(&mut self) {
        self.update_tex_if_needed();
        self.queue.write_buffer(&self.input_buffer, 0, bytemuck::cast_slice(&[self.input_uniform]));
    }

    fn update_tex_if_needed(&mut self) {
        if self.image_tex_uniform.updated {
            let diffuse_texture = self.device.create_texture(
                &wgpu::TextureDescriptor {
                    size: self.image_tex_uniform.get_texture_size(),
                    mip_level_count: 1, 
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    // Most images are stored using sRGB so we need to reflect that here.
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                    // COPY_DST means that we want to copy data to this texture
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    label: Some("dynamic_texture"),
                }
            );
            self.image_tex_uniform.diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());

            self.diffuse_bind_group = self.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout: &self.image_tex_uniform.texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&self.image_tex_uniform.diffuse_texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.image_tex_uniform.diffuse_sampler),
                        },
                    ],
                    label: Some("diffuse_bind_group"),
                }
            );

            self.image_tex_uniform.diffuse_texture = diffuse_texture;
            

            self.queue.write_texture(
                // Tells wgpu where to copy the pixel data
                self.image_tex_uniform.img_copy_tex(),
                // The actual pixel data
                &self.image_tex_uniform.get_pixels(),
                // The layout of the texture
                self.image_tex_uniform.image_data_layout(),
                self.image_tex_uniform.get_texture_size(),
            );
            self.image_tex_uniform.updated = false;
            let (width, height) = self.image_tex_uniform.get_dims();
            self.input_uniform.img_ratio = (width as f32)/ (height as f32);
            
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder")
            });
        
        {
            let r: f64 = 0.16471;
            let g: f64 = 0.08627;
            let b: f64 = 0.67843;
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: r.powf(2.2),
                            g: g.powf(2.2),
                            b: b.powf(2.2),
                            a: 1.0
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]); 
            render_pass.set_bind_group(1, &self.input_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..6, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();
        //let copy = output.texture.as_image_copy();
        Ok(())
    }


}

pub enum FrontendEvent {
    STEP,
    FILL_MODE,
    NEW_COLORS,
    NewImage(WebImage),
}

use wasm_bindgen::JsCast;
pub fn run(view_width: f64, view_height: f64) -> EventLoopProxy<FrontendEvent>{

    
    let event_loop = EventLoopBuilder::<FrontendEvent>::with_user_event().build();
    let canvas_el = web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            doc.get_element_by_id("canvas")
        })
        .unwrap();
    
    let canvas = canvas_el.dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    //let img_ratio = img.width() as f64 / img.height() as f64;
    //let view_ratio = view_width / view_height;

    let render_height: f64 = view_height;
    let render_width: f64 = view_width;
    // ratio is big when wide, small when tall
    /*if img_ratio < view_ratio { // wide display (constrain height)
        render_height = view_height;
        render_width = view_height * img_ratio;
    } else { // tall display (constrain width)
        render_height = view_width / img_ratio;
        render_width = view_width;
    }*/

    use winit::platform::web::WindowBuilderExtWebSys;

    let window = WindowBuilder::new()
        .with_canvas(Some(canvas))
        .with_prevent_default(false)
        .with_focusable(false)
        .build(&event_loop)
        .unwrap();
    use winit::dpi::LogicalSize;
    window.set_inner_size(LogicalSize::new(render_width, render_height));

    let mut state = pollster::block_on(State::new(&window));
    state.update();
    match state.render() {
        Ok(_) => {}
        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
        Err(e) => eprintln!("{:?}", e),
    }

    let proxy = event_loop.create_proxy();
    spawn_local(async move {
        event_loop.run(move |event, _, control_flow| match event {
            Event::RedrawRequested(_) => {
                // normally the render code would go here for a game
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::UserEvent(event) => { // custom event from proxy

                match event {
                    FrontendEvent::STEP => state.input_uniform.step(),
                    FrontendEvent::FILL_MODE => state.input_uniform.toggle_fill(),
                    FrontendEvent::NEW_COLORS => (),
                    FrontendEvent::NewImage(img) => state.image_tex_uniform.update_tex(img)
                }

                state.update();
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            _ => {}
        });
    });
    return proxy;
}


