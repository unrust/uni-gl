use gl;
use std::os::raw::c_void;

use std::ffi::CStr;
use std::ffi::CString;
use std::ops::Deref;
use std::ptr;
use std::str;

use crate::common::*;
use crate::glenum::*;

pub type Reference = u32;

#[derive(Debug, PartialEq, Clone)]
/// uni-gl internal OpenGL context.
///
/// You shouldn't use this struct directly. Instead, call the methods on [`WebGLRenderingContext`]
/// as it automatically dereferences into a [`GLContext`].
///
/// This doc is not intended to cover all OpenGL API in depth.
/// Check [https://www.khronos.org/opengl/](https://www.khronos.org/opengl/) for more information.
pub struct GLContext {
    /// openGL internal reference
    pub reference: Reference,
    /// whether this context is a WebGL 2.0 context
    pub is_webgl2: bool,
}

/// panics with a proper message if the last OpenGL call returned an error
pub fn check_gl_error(msg: &str) {
    unsafe {
        let err = gl::GetError();
        if err != gl::NO_ERROR {
            panic!(
                "GLError: {} {} ({})",
                msg,
                err,
                match err {
                    gl::INVALID_ENUM => "invalid enum",
                    gl::INVALID_OPERATION => "invalid operation",
                    gl::INVALID_VALUE => "invalid value",
                    gl::OUT_OF_MEMORY => "out of memory",
                    gl::STACK_OVERFLOW => "stack overflow",
                    gl::STACK_UNDERFLOW => "stack underflow",
                    _ => "unknown error",
                }
            );
        }
    }
}

/// gl::GetString convenient wrapper
fn get_string(param: u32) -> String {
    return unsafe {
        let data = CStr::from_ptr(gl::GetString(param) as *const _)
            .to_bytes()
            .to_vec();
        String::from_utf8(data).unwrap()
    };
}

pub type WebGLContext<'p> = Box<dyn 'p + for<'a> FnMut(&'a str) -> *const c_void>;

impl WebGLRenderingContext {
    /// create an OpenGL context.
    ///
    /// uni-gl should be used with the uni-app crate.
    /// You can create a [`WebGLRenderingContext`] with following code :
    /// ```ignore
    /// let app = uni_app::App::new(...);
    /// let gl = uni_gl::WebGLRenderingContext::new(app.canvas());
    /// ```
    pub fn new<'p>(mut loadfn: WebGLContext<'p>) -> WebGLRenderingContext {
        gl::load_with(move |name| loadfn(name));

        WebGLRenderingContext {
            common: GLContext::new(),
        }
    }
}

impl GLContext {
    pub fn new() -> GLContext {
        //  unsafe { gl::Enable(gl::DEPTH_TEST) };
        println!("opengl {}", get_string(gl::VERSION));
        println!(
            "shading language {}",
            get_string(gl::SHADING_LANGUAGE_VERSION)
        );
        println!("vendor {}", get_string(gl::VENDOR));
        GLContext {
            reference: 0,
            is_webgl2: true,
        }
    }

    pub fn print<T: Into<String>>(msg: T) {
        print!("{}", msg.into());
    }

    /// create a new OpenGL buffer
    pub fn create_buffer(&self) -> WebGLBuffer {
        let mut buffer = WebGLBuffer(0);
        unsafe {
            gl::GenBuffers(1, &mut buffer.0);
        }
        check_gl_error("create_buffer");
        buffer
    }

    /// delete an existing buffer
    pub fn delete_buffer(&self, buffer: &WebGLBuffer) {
        unsafe {
            gl::DeleteBuffers(1, &buffer.0);
        }
        check_gl_error("delete_buffer");
    }

    /// bind a buffer to current state.
    pub fn bind_buffer(&self, kind: BufferKind, buffer: &WebGLBuffer) {
        unsafe {
            gl::BindBuffer(kind as _, buffer.0);
        }
        check_gl_error("bind_buffer");
    }

    /// fills a buffer with data.
    ///
    /// kind : see [`GLContext::bind_buffer`].
    pub fn buffer_data(&self, kind: BufferKind, data: &[u8], draw: DrawMode) {
        unsafe {
            gl::BufferData(kind as _, data.len() as _, data.as_ptr() as _, draw as _);
        }
        check_gl_error("buffer_data");
    }

    /// update a subset of a buffer
    ///
    /// kind : see [`GLContext::bind_buffer`].
    ///
    /// offset : offset in the buffer where data replacement will begin
    pub fn buffer_sub_data(&self, kind: BufferKind, offset: u32, data: &[u8]) {
        unsafe {
            gl::BufferSubData(kind as _, offset as _, data.len() as _, data.as_ptr() as _);
        }
        check_gl_error("buffer_sub_data");
    }

    /// this buffer is not bound to the current state anymore.
    pub fn unbind_buffer(&self, kind: BufferKind) {
        unsafe {
            gl::BindBuffer(kind as _, 0);
        }
        check_gl_error("unbind_buffer");
    }

    /// create a new shader.
    pub fn create_shader(&self, kind: ShaderKind) -> WebGLShader {
        let shader = unsafe { WebGLShader(gl::CreateShader(kind as _)) };
        check_gl_error("create_shader");

        return shader;
    }

    /// set or replace the source code in a shader
    pub fn shader_source(&self, shader: &WebGLShader, source: &str) {
        let src = CString::new(source).unwrap();
        unsafe {
            gl::ShaderSource(shader.0, 1, &src.as_ptr(), ptr::null());
        }
        check_gl_error("shader_source");
    }

    /// compile a shader
    pub fn compile_shader(&self, shader: &WebGLShader) {
        unsafe {
            gl::CompileShader(shader.0);

            // Get the compile status
            let mut status = gl::FALSE as gl::types::GLint;
            gl::GetShaderiv(shader.0, gl::COMPILE_STATUS, &mut status);

            // Fail on error
            if status != (gl::TRUE as gl::types::GLint) {
                let mut len = 0;
                gl::GetShaderiv(shader.0, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                gl::GetShaderInfoLog(
                    shader.0,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut gl::types::GLchar,
                );

                match String::from_utf8(buf) {
                    Ok(s) => panic!("{}", s),
                    Err(_) => panic!("Compile shader fail, reason unknown"),
                }
            }
        }

        check_gl_error("compile_shader");
    }

    /// create a program
    pub fn create_program(&self) -> WebGLProgram {
        let p = unsafe { WebGLProgram(gl::CreateProgram()) };
        check_gl_error("create_program");
        p
    }

    /// link a program
    pub fn link_program(&self, program: &WebGLProgram) {
        unsafe {
            gl::LinkProgram(program.0);
            // Get the link status
            let mut status = gl::FALSE as gl::types::GLint;
            gl::GetProgramiv(program.0, gl::LINK_STATUS, &mut status);

            // Fail on error
            if status != (gl::TRUE as gl::types::GLint) {
                let mut len = 0;
                gl::GetProgramiv(program.0, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                gl::GetProgramInfoLog(
                    program.0,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut gl::types::GLchar,
                );

                match String::from_utf8(buf) {
                    Ok(s) => panic!("{}", s),
                    Err(_) => panic!("Link program fail, reason unknown"),
                }
            }
        }
        check_gl_error("link_program");
    }

    /// bind a program to the current state.
    pub fn use_program(&self, program: &WebGLProgram) {
        unsafe {
            gl::UseProgram(program.0);
        }
        check_gl_error("use_program");
    }

    /// attach a shader to a program. A program must have two shaders : vertex and fragment shader.
    pub fn attach_shader(&self, program: &WebGLProgram, shader: &WebGLShader) {
        unsafe {
            gl::AttachShader(program.0, shader.0);
        }
        check_gl_error("attach_shader");
    }

    /// associate a generic vertex attribute index with a named attribute
    pub fn bind_attrib_location(&self, program: &WebGLProgram, name: &str, loc: u32) {
        let c_name = CString::new(name).unwrap();
        unsafe {
            gl::BindAttribLocation(program.0 as _, loc as _, c_name.as_ptr());
            check_gl_error("bind_attrib_location");
        }
    }

    /// return the location of an attribute variable
    pub fn get_attrib_location(&self, program: &WebGLProgram, name: &str) -> Option<u32> {
        let c_name = CString::new(name).unwrap();
        unsafe {
            let location = gl::GetAttribLocation(program.0 as _, c_name.as_ptr());
            check_gl_error("get_attrib_location");
            if location == -1 {
                return None;
            }
            return Some(location as _);
        }
    }

    /// return the location of a uniform variable
    pub fn get_uniform_location(
        &self,
        program: &WebGLProgram,
        name: &str,
    ) -> Option<WebGLUniformLocation> {
        let c_name = CString::new(name).unwrap();
        unsafe {
            let location = gl::GetUniformLocation(program.0 as _, c_name.as_ptr());
            check_gl_error(&format!("get_uniform_location {}", name));
            if location == -1 {
                return None;
            }
            return Some(WebGLUniformLocation {
                reference: location as _,
                name: name.into(),
            });
        }
    }

    /// define an array of generic vertex attribute data
    pub fn vertex_attrib_pointer(
        &self,
        location: u32,
        size: AttributeSize,
        kind: DataType,
        normalized: bool,
        stride: u32,
        offset: u32,
    ) {
        unsafe {
            gl::VertexAttribPointer(
                location as _,
                size as _,
                kind as _,
                normalized as _,
                stride as _,
                offset as _,
            );
        }
        // println!(
        //     "{:?} {:?} {:?} {:?} {:?} {:?} {:?}",
        //     location, size, kind, kind as u32, normalized, stride, offset
        // );
        check_gl_error("vertex_attrib_pointer");
    }

    /// enable a generic vertex attribute array
    pub fn enable_vertex_attrib_array(&self, location: u32) {
        unsafe {
            gl::EnableVertexAttribArray(location as _);
        }
        check_gl_error("enable_vertex_attrib_array");
    }

    /// specify clear values for the color buffers
    pub fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            gl::ClearColor(r, g, b, a);
        }
        check_gl_error("clear_color");
    }

    /// enable GL capabilities.
    ///
    /// flag should be one of [`Flag`]
    pub fn enable(&self, flag: i32) {
        unsafe {
            gl::Enable(flag as _);
        }
        check_gl_error("enable");
    }

    /// disable GL capabilities.
    ///
    /// flag should be one of [`Flag`]
    pub fn disable(&self, flag: i32) {
        unsafe {
            gl::Disable(flag as _);
        }
        check_gl_error("disable");
    }

    /// specify whether front- or back-facing polygons can be culled
    pub fn cull_face(&self, flag: Culling) {
        unsafe {
            gl::CullFace(flag as _);
        }
        check_gl_error("cullface");
    }

    /// enable or disable writing into the depth buffer
    pub fn depth_mask(&self, b: bool) {
        unsafe {
            gl::DepthMask(b as _);
        }
        check_gl_error("depth_mask");
    }

    /// specify the value used for depth buffer comparisons
    pub fn depth_func(&self, d: DepthTest) {
        unsafe {
            gl::DepthFunc(d as _);
        }

        check_gl_error("depth_func");
    }

    /// specify the clear value for the depth buffer
    pub fn clear_depth(&self, value: f32) {
        unsafe {
            gl::ClearDepth(value as _);
        }
        check_gl_error("clear_depth");
    }

    /// clear buffers to preset values
    pub fn clear(&self, bit: BufferBit) {
        unsafe {
            gl::Clear(bit as _);
        }
        check_gl_error("clear");
    }

    /// set the viewport
    pub fn viewport(&self, x: i32, y: i32, width: u32, height: u32) {
        unsafe {
            gl::Viewport(x, y, width as _, height as _);
        };
        check_gl_error("viewport");
    }

    /// render primitives from indexed array data
    pub fn draw_elements(&self, mode: Primitives, count: usize, kind: DataType, offset: u32) {
        unsafe {
            gl::DrawElements(mode as _, count as _, kind as _, offset as _);
        };
        check_gl_error("draw_elements");
    }

    /// render primitives from array data
    pub fn draw_arrays(&self, mode: Primitives, count: usize) {
        unsafe {
            gl::DrawArrays(mode as _, 0, count as _);
        };
        check_gl_error("draw_arrays");
    }

    /// read a block of pixels from the frame buffer
    pub fn read_pixels(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        format: PixelFormat,
        kind: PixelType,
        data: &mut [u8],
    ) {
        unsafe {
            gl::ReadPixels(
                x as _,
                y as _,
                width as _,
                height as _,
                format as _,
                kind as _,
                data.as_mut_ptr() as _,
            );
            check_gl_error("read_pixels");
        }
    }

    /// set pixel storage modes
    pub fn pixel_storei(&self, storage: PixelStorageMode, value: i32) {
        unsafe {
            gl::PixelStorei(storage as _, value);
            check_gl_error("pixel_storei");
        }
    }

    /// specify a two-dimensional texture image
    pub fn tex_image2d(
        &self,
        target: TextureBindPoint,
        level: u8,
        width: u16,
        height: u16,
        format: PixelFormat,
        kind: PixelType,
        pixels: &[u8],
    ) {
        let p: *const c_void;

        if pixels.len() > 0 {
            p = pixels.as_ptr() as _;
        } else {
            p = 0 as _;
        }

        unsafe {
            gl::TexImage2D(
                target as _,
                level as _,
                format as _, // internal format
                width as _,
                height as _,
                0,
                format as _, // format
                kind as _,
                p as _,
            );
        }

        check_gl_error("tex_image2d");
    }

    /// update a part of a two-dimensional texture subimage
    pub fn tex_sub_image2d(
        &self,
        target: TextureBindPoint,
        level: u8,
        xoffset: u16,
        yoffset: u16,
        width: u16,
        height: u16,
        format: PixelFormat,
        kind: PixelType,
        pixels: &[u8],
    ) {
        unsafe {
            gl::TexSubImage2D(
                target as _,
                level as _,
                xoffset as _,
                yoffset as _,
                width as _,
                height as _,
                format as _,
                kind as _,
                pixels.as_ptr() as _,
            );
        }

        check_gl_error("tex_sub_image2d");
    }

    /// specify a two-dimensional texture image in a compressed format
    pub fn compressed_tex_image2d(
        &self,
        target: TextureBindPoint,
        level: u8,
        compression: TextureCompression,
        width: u16,
        height: u16,
        data: &[u8],
    ) {
        unsafe {
            gl::CompressedTexImage2D(
                target as _,
                level as _,
                compression as _,
                width as _,
                height as _,
                0,
                data.len() as _, //gl::UNSIGNED_BYTE as _,
                data.as_ptr() as _,
            );
        }

        check_gl_error("compressed_tex_image2d");
    }

    /// return informations about current program
    pub fn get_program_parameter(&self, program: &WebGLProgram, pname: ShaderParameter) -> i32 {
        let mut res = 0;
        unsafe {
            gl::GetProgramiv(program.0, pname as _, &mut res);
        }

        check_gl_error("get_program_parameter");
        res
    }

    // pub fn get_active_uniform(&self, program: &WebGLProgram, location: u32) -> WebGLActiveInfo {
    //     let mut name: Vec<u8> = Vec::with_capacity(NAME_SIZE);
    //     let mut size = 0i32;
    //     let mut len = 0i32;
    //     let mut kind = 0u32;

    //     unsafe {
    //         gl::GetActiveUniform(
    //             program.0,
    //             location as _,
    //             NAME_SIZE as _,
    //             &mut len,
    //             &mut size,
    //             &mut kind,
    //             name.as_mut_ptr() as _,
    //         );
    //         name.set_len(len as _);
    //     };

    //     use std::mem;

    //     WebGLActiveInfo::new(
    //         String::from_utf8(name).unwrap(),
    //         //location as _,
    //         size as _,
    //         unsafe { mem::transmute::<u16, UniformType>(kind as _) },
    //         0
    //         //unsafe { mem::transmute::<u16, DataType>(kind as _) },
    //     )
    // }

    // pub fn get_active_attrib(&self, program: &WebGLProgram, location: u32) -> WebGLActiveInfo {
    //     let mut name: Vec<u8> = Vec::with_capacity(NAME_SIZE);
    //     let mut size = 0i32;
    //     let mut len = 0i32;
    //     let mut kind = 0u32;

    //     unsafe {
    //         gl::GetActiveAttrib(
    //             program.0,
    //             location as _,
    //             NAME_SIZE as _,
    //             &mut len,
    //             &mut size,
    //             &mut kind,
    //             name.as_mut_ptr() as _,
    //         );
    //         name.set_len(len as _);
    //     }
    //     println!("name {:?}", name);
    //     use std::mem;
    //     //let c_name = unsafe { CString::from_raw(name[0..(len+1)].as_mut_ptr())};
    //     WebGLActiveInfo::new(
    //         String::from_utf8(name).expect("utf8 parse failed"),
    //         //location,
    //         size as _,
    //         //DataType::Float
    //         unsafe { mem::transmute::<u16, UniformType>(kind as _) },
    //         0,
    //     )
    // }

    /// create a new texture object
    pub fn create_texture(&self) -> WebGLTexture {
        let mut handle = WebGLTexture(0);
        unsafe {
            gl::GenTextures(1, &mut handle.0);
        }
        check_gl_error("create_texture");

        handle
    }

    /// destroy a texture object
    pub fn delete_texture(&self, texture: &WebGLTexture) {
        unsafe {
            gl::DeleteTextures(1, texture.0 as _);
        }

        check_gl_error("delete_texture");
    }

    /// generate mipmaps for current 2D texture
    pub fn generate_mipmap(&self) {
        unsafe {
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        check_gl_error("generate_mipmap");
    }

    /// generate mipmaps for current cube map texture
    pub fn generate_mipmap_cube(&self) {
        unsafe {
            gl::GenerateMipmap(gl::TEXTURE_CUBE_MAP);
        }

        check_gl_error("generate_mipmap_cube");
    }

    /// select active texture unit
    pub fn active_texture(&self, active: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + active);
        }

        check_gl_error("active_texture");
    }

    /// bind a named 2D texture to a texturing target
    pub fn bind_texture(&self, texture: &WebGLTexture) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, texture.0);
        }

        check_gl_error("bind_texture");
    }

    /// current 2D texture is not bound to current state anymore
    pub fn unbind_texture(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        check_gl_error("unbind_texture");
    }

    /// bind a named cube map texture to a texturing target
    pub fn bind_texture_cube(&self, texture: &WebGLTexture) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, texture.0);
        }

        check_gl_error("bind_texture_cube");
    }

    /// current cube map texture is not bound to current state anymore
    pub fn unbind_texture_cube(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0);
        }

        check_gl_error("unbind_texture_cube");
    }

    /// set the RGB alpha blend equation
    pub fn blend_equation(&self, eq: BlendEquation) {
        unsafe {
            gl::BlendEquation(eq as _);
        }

        check_gl_error("blend_equation");
    }

    /// specify pixel arithmetic for RGB and alpha components separately
    pub fn blend_func(&self, b1: BlendMode, b2: BlendMode) {
        unsafe {
            gl::BlendFunc(b1 as _, b2 as _);
        }

        check_gl_error("blend_func");
    }

    /// set the blend color
    pub fn blend_color(&self, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            gl::BlendColor(r, g, b, a);
        }

        check_gl_error("blend_color");
    }

    /// specify the value of a mat4 uniform variable for the current program object
    pub fn uniform_matrix_4fv(&self, location: &WebGLUniformLocation, value: &[[f32; 4]; 4]) {
        unsafe {
            gl::UniformMatrix4fv(*location.deref() as i32, 1, false as _, &value[0] as _);
        }
        check_gl_error("uniform_matrix_4fv");
    }

    /// specify the value of a mat3 uniform variable for the current program object
    pub fn uniform_matrix_3fv(&self, location: &WebGLUniformLocation, value: &[[f32; 3]; 3]) {
        unsafe {
            gl::UniformMatrix3fv(*location.deref() as i32, 1, false as _, &value[0] as _);
        }
        check_gl_error("uniform_matrix_3fv");
    }

    /// specify the value of a mat2 uniform variable for the current program object
    pub fn uniform_matrix_2fv(&self, location: &WebGLUniformLocation, value: &[[f32; 2]; 2]) {
        unsafe {
            gl::UniformMatrix2fv(*location.deref() as i32, 1, false as _, &value[0] as _);
        }
        check_gl_error("uniform_matrix_2fv");
    }

    /// specify the value of an int uniform variable for the current program object
    pub fn uniform_1i(&self, location: &WebGLUniformLocation, value: i32) {
        unsafe {
            gl::Uniform1i(*location.deref() as i32, value as _);
        }
        check_gl_error("uniform_1i");
    }

    /// specify the value of a float uniform variable for the current program object
    pub fn uniform_1f(&self, location: &WebGLUniformLocation, value: f32) {
        unsafe {
            gl::Uniform1f(*location.deref() as i32, value as _);
        }
        check_gl_error("uniform_1f");
    }

    /// specify the value of a vec2 uniform variable for the current program object
    pub fn uniform_2f(&self, location: &WebGLUniformLocation, value: (f32, f32)) {
        unsafe {
            gl::Uniform2f(*location.deref() as _, value.0, value.1);
        }
        check_gl_error("uniform_2f");
    }

    /// specify the value of a vec3 uniform variable for the current program object
    pub fn uniform_3f(&self, location: &WebGLUniformLocation, value: (f32, f32, f32)) {
        unsafe {
            gl::Uniform3f(*location.deref() as _, value.0, value.1, value.2);
        }
        check_gl_error("uniform_3f");
    }

    /// specify the value of a vec4 uniform variable for the current program object
    pub fn uniform_4f(&self, location: &WebGLUniformLocation, value: (f32, f32, f32, f32)) {
        unsafe {
            gl::Uniform4f(*location.deref() as _, value.0, value.1, value.2, value.3);
        }
        check_gl_error("uniform_4f");
    }

    /// set texture integer parameters
    pub fn tex_parameteri(&self, kind: TextureKind, pname: TextureParameter, param: i32) {
        unsafe {
            gl::TexParameteri(kind as _, pname as _, param);
        }
        check_gl_error("tex_parameteri");
    }

    /// set texture float parameters
    pub fn tex_parameterfv(&self, kind: TextureKind, pname: TextureParameter, param: f32) {
        unsafe {
            gl::TexParameterfv(kind as _, pname as _, &param);
        }
        check_gl_error("tex_parameterfv");
    }

    /// create a vertex array object
    pub fn create_vertex_array(&self) -> WebGLVertexArray {
        let mut vao = WebGLVertexArray(0);
        unsafe {
            gl::GenVertexArrays(1, &mut vao.0);
        }
        check_gl_error("create_vertex_array");
        vao
    }

    /// destroy a vertex array object
    pub fn delete_vertex_array(&self, vao: &WebGLVertexArray) {
        unsafe {
            gl::DeleteVertexArrays(1, &vao.0);
        }
        check_gl_error("delete_vertex_array");
    }

    /// bind a vertex array object to current state
    pub fn bind_vertex_array(&self, vao: &WebGLVertexArray) {
        unsafe {
            gl::BindVertexArray(vao.0);
        }
        check_gl_error("bind_vertex_array");
    }

    /// current vertex array object is not bound to the current state anymore
    pub fn unbind_vertex_array(&self, _vao: &WebGLVertexArray) {
        unsafe {
            gl::BindVertexArray(0);
        }
        check_gl_error("unbind_vertex_array");
    }

    /// specify which color buffers are to be drawn into
    pub fn draw_buffer(&self, buffers: &[ColorBuffer]) {
        unsafe {
            for value in buffers {
                gl::DrawBuffer(*value as _);
            }
        }
        check_gl_error("draw_buffer");
    }

    /// create a new framebuffer
    pub fn create_framebuffer(&self) -> WebGLFrameBuffer {
        let mut fb = WebGLFrameBuffer(0);
        unsafe {
            gl::GenFramebuffers(1, &mut fb.0);
        }
        check_gl_error("create_framebuffer");
        fb
    }

    /// destroy a framebuffer
    pub fn delete_framebuffer(&self, fb: &WebGLFrameBuffer) {
        unsafe {
            gl::DeleteFramebuffers(1, &fb.0);
        }
        check_gl_error("delete_framebuffer");
    }

    /// bind a framebuffer to the current state
    pub fn bind_framebuffer(&self, buffer: Buffers, fb: &WebGLFrameBuffer) {
        unsafe {
            gl::BindFramebuffer(buffer as u32, fb.0);
        }

        check_gl_error("bind_framebuffer");
    }

    /// attach a texture to a framebuffer
    pub fn framebuffer_texture2d(
        &self,
        target: Buffers,
        attachment: Buffers,
        textarget: TextureBindPoint,
        texture: &WebGLTexture,
        level: i32,
    ) {
        unsafe {
            gl::FramebufferTexture2D(
                target as u32,
                attachment as u32,
                textarget as u32,
                texture.0,
                level,
            );
        }

        check_gl_error("framebuffer_texture2d");
    }

    /// unbind a framebuffer
    pub fn unbind_framebuffer(&self, buffer: Buffers) {
        unsafe {
            gl::BindFramebuffer(buffer as u32, 0);
        }

        check_gl_error("unbind_framebuffer");
    }
}
