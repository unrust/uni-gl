extern crate uni_app;
extern crate uni_gl;

use std::mem::size_of;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    main();
    Ok(())
}

// helper to easily convert rust vectors into &[u8] needed by opengl
trait IntoBytes {
    fn into_bytes(self) -> Vec<u8>;
}

impl<T> IntoBytes for Vec<T> {
    fn into_bytes(self) -> Vec<u8> {
        let len = size_of::<T>() * self.len();
        unsafe {
            let slice = self.into_boxed_slice();
            Vec::<u8>::from_raw_parts(Box::into_raw(slice) as _, len, len)
        }
    }
}

fn main() {
    // create the game window (native) or canvas (web)
    let app = uni_app::App::new(uni_app::AppConfig {
        size: (800, 600),
        title: "my game".to_owned(),
        vsync: true,
        show_cursor: true,
        headless: false,
        resizable: true,
        fullscreen: false,
        intercept_close_request: false,
    });
    // retrieve the opengl/webgl/webgl2 context
    let gl = uni_gl::WebGLRenderingContext::new(app.canvas());

    // now do some opengl stuff
    // version of the shader depends if we are native or embedded
    let version = if uni_gl::IS_GL_ES { "300 es" } else { "150" };
    let vert_shader = compile_shader(
        &gl,
        uni_gl::ShaderKind::Vertex,
        &format!(
            r##"#version {version}
            in vec4 position;
            void main() {{
                gl_Position = position;
            }}
        "##
        ),
    );

    let frag_shader = compile_shader(
        &gl,
        uni_gl::ShaderKind::Fragment,
        &format!(
            r##"#version {version}
            precision mediump float;
            uniform float time;
            out vec4 FragColor;
            void main() {{
                float r=(cos(time)+1.0)*0.5;
                float g=(sin(time)+1.0)*0.5;
                FragColor = vec4(r,g,0.5,1);
            }}
        "##,
        ),
    );
    let program = link_program(&gl, &vert_shader, &frag_shader);
    gl.use_program(&program);

    let vertices: Vec<f32> = vec![-0.7, -0.7, 0.0, 0.7, -0.7, 0.0, 0.0, 0.7, 0.0];
    let vert_count = vertices.len() / 3;
    let position_attribute_location = gl.get_attrib_location(&program, "position").unwrap();
    let buffer = gl.create_buffer();
    gl.bind_buffer(uni_gl::BufferKind::Array, &buffer);
    gl.buffer_data(
        uni_gl::BufferKind::Array,
        &vertices.into_bytes(),
        uni_gl::DrawMode::Static,
    );
    let vao = gl.create_vertex_array();
    gl.bind_vertex_array(&vao);
    gl.vertex_attrib_pointer(
        position_attribute_location,
        uni_gl::AttributeSize::Three,
        uni_gl::DataType::Float,
        false,
        0,
        0,
    );
    gl.enable_vertex_attrib_array(position_attribute_location);
    gl.bind_vertex_array(&vao);

    // start game loop
    app.run(move |_app: &mut uni_app::App| {
        let now = uni_app::now();
        let time = gl.get_uniform_location(&program, "time").unwrap();
        gl.uniform_1f(&time, now as f32);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(uni_gl::BufferBit::Color);
        // render a triangle
        gl.draw_arrays(uni_gl::Primitives::Triangles, vert_count);
    });
}

pub fn compile_shader(
    gl: &uni_gl::WebGLRenderingContext,
    shader_type: uni_gl::ShaderKind,
    source: &str,
) -> uni_gl::WebGLShader {
    let shader = gl.create_shader(shader_type);
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    shader
}

pub fn link_program(
    gl: &uni_gl::WebGLRenderingContext,
    vert_shader: &uni_gl::WebGLShader,
    frag_shader: &uni_gl::WebGLShader,
) -> uni_gl::WebGLProgram {
    let program = gl.create_program();
    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);
    program
}
