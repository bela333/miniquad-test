use std::{collections::HashSet, time::Instant};

use miniquad::{*, gl::glLinkProgram};
use cgmath::{Vector3, Vector4, Vector2, vec2, vec4, Matrix4, SquareMatrix, vec3, Perspective, perspective, Deg, Point3, point3, Matrix3, Transform, EuclideanSpace};
use shader::Uniforms;

#[repr(C)]
struct Vertex {
    pos: Vector3<f32>,
    color: Vector4<f32>,
}

struct Stage {
    pipeline: Pipeline,
    bindings: Bindings,
    ctx: Box<dyn RenderingBackend>,
    perspective: Matrix4<f32>,
    camera_pos: Point3<f32>,
    camera_target: Point3<f32>,
    world_mat: Matrix4<f32>,
    keys_down: HashSet<KeyCode>,
    last_frame: Instant
}

impl Stage {
    pub fn new() -> Stage {
        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        #[rustfmt::skip]
        let vertices: [Vertex; 3] = [
            Vertex { pos : vec3(-0.5, -0.5, 0.0), color: vec4(1., 0., 0., 1.) },
            Vertex { pos : vec3( 0.5, -0.5, 0.0), color: vec4(0., 1., 0., 1.) },
            Vertex { pos : vec3( 0.0,  0.5, 0.0), color: vec4(0., 0., 1., 1.) },
        ];
        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );

        let indices: [u16; 3] = [0, 1, 2];
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
        );

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer: index_buffer,
            images: vec![],
        };

        let shader = ctx
            .new_shader(
                match ctx.info().backend {
                    Backend::OpenGl => ShaderSource::Glsl {
                        vertex: shader::VERTEX,
                        fragment: shader::FRAGMENT,
                    },
                    _ => unreachable!()
                },
                shader::meta(),
            )
            .unwrap_or_else(|err|{
                match err {
                    ShaderError::CompilationError { shader_type, error_message } => {
                        println!("A {:?} error has occured:", shader_type);
                        println!("{}", error_message);
                        panic!()
                    },
                    _ => panic!("{:?}", err)
                }
            });

        let pipeline = ctx.new_pipeline_with_params(
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("in_pos", VertexFormat::Float3),
                VertexAttribute::new("in_color", VertexFormat::Float4),
            ],
            shader,
            PipelineParams{
                depth_write: true,
                depth_test: Comparison::LessOrEqual,
                cull_face: CullFace::Back,
                ..Default::default()
            }
        );

        let screen_size = window::screen_size();

        Stage {
            pipeline,
            bindings,
            ctx,
            camera_pos: point3(0.0, 0.0, 1.0),
            camera_target: point3(0.0, 0.0, 0.0),
            perspective: perspective(Deg(90.0), screen_size.0/screen_size.1, 0.1, 100.0),
            world_mat: Matrix4::identity(),
            keys_down: HashSet::new(),
            last_frame: Instant::now()
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {

        let delta_time = self.last_frame.elapsed();
        self.last_frame = Instant::now();

        if self.keys_down.contains(&KeyCode::W) {
            self.camera_pos.z -= delta_time.as_secs_f32();
        }

        if self.keys_down.contains(&KeyCode::A) {
            self.camera_pos.x -= delta_time.as_secs_f32();
        }

        if self.keys_down.contains(&KeyCode::S) {
            self.camera_pos.z += delta_time.as_secs_f32();
        }

        if self.keys_down.contains(&KeyCode::D) {
            self.camera_pos.x += delta_time.as_secs_f32();
        }

        let look_at = Matrix3::look_at_rh(self.camera_pos, self.camera_target, vec3(0.0, 1.0, 0.0));
        let translate  = Matrix4::from_translation(-self.camera_pos.to_vec());
        let look_at: Matrix4<f32> = look_at.into();
        self.world_mat = look_at*translate;



        
        
    }

    fn key_down_event(&mut self, _keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        if _repeat {
            return;
        }
        self.keys_down.insert(_keycode);
    }

    fn key_up_event(&mut self, _keycode: KeyCode, _keymods: KeyMods) {
        self.keys_down.remove(&_keycode);
    }

    fn draw(&mut self) {
        self.ctx.begin_default_pass(PassAction::Clear { color: Some((0.0, 0.0, 0.0, 1.0)), depth: Some(1.0), stencil: None});

        
        self.ctx.apply_pipeline(&self.pipeline);
        self.ctx.apply_bindings(&self.bindings);
        
        let uniforms = Uniforms{
            perspective: self.perspective,
            view: self.world_mat,
            world: [
                Matrix4::from_translation(vec3(0.0, 0.0, -0.3)),
                Matrix4::from_translation(vec3(0.0, 0.0, -0.5))
            ]
        };
        self.ctx.apply_uniforms(UniformsSource::table(&uniforms));

        self.ctx.draw(0, 3, 2);

        self.ctx.end_render_pass();

        self.ctx.commit_frame();
    }
}

fn main() {
    let mut conf = conf::Conf::default();
    conf.platform.apple_gfx_api = conf::AppleGfxApi::OpenGl;

    miniquad::start(conf, move || Box::new(Stage::new()));
}

mod shader {
    use cgmath::Matrix4;
    use miniquad::*;

    pub const VERTEX: &str = include_str!("shaders/instanced.vert");

    pub const FRAGMENT: &str = include_str!("shaders/basic.frag");

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec![],
            uniforms: UniformBlockLayout { uniforms: vec![
                UniformDesc{array_count: 1, name: "perspective".to_owned(), uniform_type: UniformType::Mat4},
                UniformDesc{array_count: 1, name: "view".to_owned(), uniform_type: UniformType::Mat4},
                UniformDesc{array_count: 2, name: "world".to_owned(), uniform_type: UniformType::Mat4}
            ] },
        }
    }
    #[repr(C)]
    pub struct Uniforms{
        pub perspective: Matrix4<f32>,
        pub view: Matrix4<f32>,
        pub world: [Matrix4<f32>; 2]
    }
}