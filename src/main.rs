use std::{collections::HashSet, time::Instant};

use miniquad::{*};
use cgmath::{Vector3, Vector4, vec4, Matrix4, SquareMatrix, vec3, perspective, Deg, Point3, point3, Matrix3, EuclideanSpace, Rad, Basis3, Rotation3};
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
    view: Matrix4<f32>,
    keys_down: HashSet<KeyCode>,
    last_frame: Instant,
    rotate_x: f32,
    rotate_y: f32,
    fov: f32,
    near: f32,
    far: f32
}

impl Stage {
    pub fn new() -> Stage {
        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        window::show_mouse(false);
        window::set_cursor_grab(true);

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
            index_buffer,
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

        let fov = 80.0;
        let near = 0.1;
        let far = 100.0;

        Stage {
            pipeline,
            bindings,
            ctx,
            camera_pos: point3(0.0, 0.0, 1.0),
            perspective: perspective(Deg(fov), screen_size.0/screen_size.1, near, far),
            view: Matrix4::identity(),
            keys_down: HashSet::new(),
            last_frame: Instant::now(),
            rotate_x: 0.0,
            rotate_y: 0.0,
            fov,
            near,
            far,
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {

        let delta_time = self.last_frame.elapsed();
        self.last_frame = Instant::now();

        let forward = vec3(-self.rotate_y.sin(), 0.0, -self.rotate_y.cos());
        let right = vec3(self.rotate_y.cos(), 0.0, -self.rotate_y.sin());

        if self.keys_down.contains(&KeyCode::W) {
            self.camera_pos += forward*delta_time.as_secs_f32();
        }

        if self.keys_down.contains(&KeyCode::A) {
            self.camera_pos += -right*delta_time.as_secs_f32();
        }

        if self.keys_down.contains(&KeyCode::S) {
            self.camera_pos += -forward*delta_time.as_secs_f32();
        }

        if self.keys_down.contains(&KeyCode::D) {
            self.camera_pos += right*delta_time.as_secs_f32();
        }

        let rotate = Basis3::from_angle_y(Rad(self.rotate_y))*Basis3::from_angle_x(Rad(self.rotate_x));
        let rotate: Matrix3<f32> = rotate.into();
        let rotate: Matrix4<f32> = rotate.into();
        let translate  = Matrix4::from_translation(self.camera_pos.to_vec());
        self.view = (translate*rotate).invert().unwrap();



        
        
    }

    fn key_down_event(&mut self, _keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        if _repeat {
            return;
        }
        match _keycode{
            KeyCode::Escape => {
                window::quit();
            }
            _ => ()
        }
        self.keys_down.insert(_keycode);
    }

    fn key_up_event(&mut self, _keycode: KeyCode, _keymods: KeyMods) {
        self.keys_down.remove(&_keycode);
    }

    fn resize_event(&mut self, width: f32, height: f32) {
        self.perspective = perspective(Deg(self.fov), width/height, self.near, self.far);
    }

    fn raw_mouse_motion(&mut self, dx: f32, dy: f32) {
        println!("{}, {}", dx, dy);
        self.rotate_x += -dy*0.01;
        self.rotate_y += -dx*0.01;
    }

    fn draw(&mut self) {
        self.ctx.begin_default_pass(PassAction::Clear { color: Some((0.0, 0.0, 0.0, 1.0)), depth: Some(1.0), stencil: None});

        
        self.ctx.apply_pipeline(&self.pipeline);
        self.ctx.apply_bindings(&self.bindings);
        
        let uniforms = Uniforms{
            perspective: self.perspective,
            view: self.view,
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