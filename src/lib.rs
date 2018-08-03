#![recursion_limit = "512"]

#[cfg(not(target_arch = "wasm32"))]
extern crate gl;

#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate stdweb;

#[cfg(target_arch = "wasm32")]
#[path = "webgl.rs"]
pub mod webgl;

#[cfg(not(target_arch = "wasm32"))]
#[path = "webgl_native.rs"]
mod webgl;

#[cfg(not(target_arch = "wasm32"))]
/// whether current OpenGL context is OpenGL ES (Embedded System)
pub const IS_GL_ES: bool = false;

#[cfg(target_arch = "wasm32")]
pub const IS_GL_ES: bool = true;

mod glenum;

pub use glenum::*;
pub use webgl::{GLContext, WebGLContext};

pub mod common {
    use std::ops::Deref;

    type Reference = super::webgl::Reference;
    type GLContext = super::GLContext;

    #[derive(Debug, Clone)]
    /// The OpenGL rendering context. This is the struct providing most of the OpenGL API.
    pub struct WebGLRenderingContext {
        pub common: GLContext,
    }

    impl From<GLContext> for Reference {
        fn from(w: GLContext) -> Reference {
            w.reference
        }
    }

    impl Deref for WebGLRenderingContext {
        type Target = GLContext;
        fn deref(&self) -> &GLContext {
            &self.common
        }
    }

    #[derive(Debug)]
    /// an OpenGL buffer created with [`GLContext::create_buffer`].
    ///
    /// Buffers are used to store vertex attributes
    /// (position, normal, uv coordinates) and indexes for indexed rendering,
    pub struct WebGLBuffer(pub Reference);

    impl Deref for WebGLBuffer {
        type Target = Reference;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Debug)]
    /// an OpenGL shader created with [`GLContext::create_shader`]
    pub struct WebGLShader(pub Reference);
    impl Deref for WebGLShader {
        type Target = Reference;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Debug, PartialEq)]
    /// an OpenGL shader created with [`GLContext::create_shader`].
    ///
    /// There are two kinds of shaders ([`ShaderKind`]) : vertex and fragment
    pub struct WebGLProgram(pub Reference);
    impl Deref for WebGLProgram {
        type Target = Reference;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Debug, PartialEq)]
    /// an OpenGL program created with [`GLContext::create_program`].
    ///
    /// It is built with a vertex shader and a fragment shader.
    pub struct WebGLTexture(pub Reference);
    impl Deref for WebGLTexture {
        type Target = Reference;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Debug)]
    /// an OpenGL vertex array object created with [`GLContext::create_vertex_array`].
    ///
    /// Vertex array objects store all the state needed to supply vertex data.
    pub struct WebGLVertexArray(pub Reference);
    impl Deref for WebGLVertexArray {
        type Target = Reference;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Debug, PartialEq)]
    /// the reference to a uniform (global GLSL variable) inside a shader, obtained with [`GLContext::get_uniform_location`].
    pub struct WebGLUniformLocation {
        pub reference: Reference,
        pub name: String,
    }
    impl Deref for WebGLUniformLocation {
        type Target = Reference;
        fn deref(&self) -> &Reference {
            &self.reference
        }
    }

    #[derive(Debug)]
    /// an OpenGL Framebuffer created with [`GLContext::create_framebuffer`].
    ///
    /// This is a special type of buffer that can be used as destination for rendering.
    pub struct WebGLFrameBuffer(pub Reference);
    impl Deref for WebGLFrameBuffer {
        type Target = Reference;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    /// Utility function to print messages to stdout (native) or the js console (web)
    pub fn print(s: &str) {
        GLContext::print(s);
    }
}

pub use self::common::*;
