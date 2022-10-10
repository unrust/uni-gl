use std::cell::RefCell;
use std::collections::HashMap;

use js_sys::{Array, Object, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

use crate::common::*;
use crate::glenum::*;

pub type Reference = i32;

macro_rules! gl_call {
    ($gl:expr, $func:ident, $($params:expr),*) => {{
        match $gl {
            WebContext::Gl2(gl) => gl.$func($($params,)*),
            WebContext::Gl(gl) => gl.$func($($params,)*),
        }
    }};
    ($gl:expr, $func:ident) => {{
        match $gl {
            WebContext::Gl2(gl) => gl.$func(),
            WebContext::Gl(gl) => gl.$func(),
        }
    }};
}

#[derive(Debug, PartialEq, Clone)]
pub enum WebContext {
    Gl2(web_sys::WebGl2RenderingContext),
    Gl(web_sys::WebGlRenderingContext),
}

#[derive(Debug, PartialEq, Clone)]
pub struct GLContext {
    pub gl: WebContext,
    pub is_webgl2: bool,
    dict: RefCell<HashMap<i32, JsValue>>,
    seq: RefCell<i32>,
}

pub type WebGLContext<'a> = &'a HtmlCanvasElement;

impl WebGLRenderingContext {
    pub fn new(canvas: WebGLContext) -> WebGLRenderingContext {
        WebGLRenderingContext {
            common: GLContext::new(&canvas.clone().into()),
        }
    }
}

impl GLContext {
    #[inline]
    pub fn log<T: Into<String>>(&self, msg: T) {
        let msg: String = msg.into();
        web_sys::console::log_1(&msg.into());
    }

    pub fn print<T: Into<String>>(msg: T) {
        let msg: String = msg.into();
        web_sys::console::log_1(&msg.into());
    }

    // utilities to store and retrieve js objects as u32
    fn add(&self, val: JsValue) -> i32 {
        let id = *self.seq.borrow();
        *self.seq.borrow_mut() = id + 1;
        self.dict.borrow_mut().insert(id, val);
        id
    }
    fn get(&self, id: i32) -> Option<JsValue> {
        self.dict.borrow().get(&id).map(|o| o.clone())
    }
    fn remove(&self, id: i32) {
        self.dict.borrow_mut().remove(&id);
    }

    pub fn new<'a>(canvas: &HtmlCanvasElement) -> GLContext {
        let gl_attribs = Object::new();
        Reflect::set(&gl_attribs, &JsValue::from_str("alpha"), &JsValue::FALSE).unwrap();
        Reflect::set(
            &gl_attribs,
            &JsValue::from_str("preserveDrawingBuffer"),
            &JsValue::TRUE,
        )
        .unwrap();
        if let Ok(gl) = canvas
            .get_context_with_context_options("webgl2", &gl_attribs)
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::WebGl2RenderingContext>()
        {
            let context = GLContext {
                gl: WebContext::Gl2(gl),
                is_webgl2: true,
                dict: RefCell::new(HashMap::new()),
                seq: RefCell::new(1),
            };
            context.display_gl_info();
            return context;
        }
        if let Ok(gl) = canvas
            .get_context_with_context_options("webgl", &gl_attribs)
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::WebGlRenderingContext>()
        {
            let context = GLContext {
                gl: WebContext::Gl(gl),
                is_webgl2: false,
                dict: RefCell::new(HashMap::new()),
                seq: RefCell::new(1),
            };
            context.display_gl_info();
            return context;
        }
        panic!("No webgl context found");
    }

    fn get_parameter(&self, id: u32) -> String {
        gl_call!(&self.gl, get_parameter, id)
            .unwrap()
            .as_string()
            .unwrap()
    }

    fn get_extension(&self, ext_name: &str) -> bool {
        gl_call!(&self.gl, get_extension, ext_name)
            .unwrap()
            .is_some()
    }

    fn display_gl_info(&self) {
        self.get_extension("WEBGL_depth_texture");
        print(&format!(
            "opengl {}",
            self.get_parameter(web_sys::WebGl2RenderingContext::VERSION)
        ));
        print(&format!(
            "shading language {}",
            self.get_parameter(web_sys::WebGl2RenderingContext::SHADING_LANGUAGE_VERSION)
        ));
        print(&format!(
            "vendor {}",
            self.get_parameter(web_sys::WebGl2RenderingContext::VENDOR)
        ));
    }

    pub fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        gl_call!(&self.gl, clear_color, r, g, b, a);
    }

    pub fn clear(&self, bit: BufferBit) {
        gl_call!(&self.gl, clear, bit as u32);
    }

    pub fn compile_shader(&self, shader: &WebGLShader) {
        let shader: web_sys::WebGlShader = self.get(shader.0).unwrap().into();
        gl_call!(&self.gl, compile_shader, &shader);
        let compiled = gl_call!(
            &self.gl,
            get_shader_parameter,
            &shader,
            web_sys::WebGl2RenderingContext::COMPILE_STATUS
        );
        if !compiled {
            print("Error in shader compilation :");
            print(&format!(
                "{}",
                gl_call!(&self.gl, get_shader_info_log, &shader).unwrap(),
            ));
        }
    }

    pub fn use_program(&self, program: &WebGLProgram) {
        let program: web_sys::WebGlProgram = self.get(program.0).unwrap().into();
        gl_call!(&self.gl, use_program, Some(&program));
    }

    pub fn get_attrib_location(&self, program: &WebGLProgram, name: &str) -> Option<u32> {
        let program: web_sys::WebGlProgram = self.get(program.0).unwrap().into();
        let loc = gl_call!(&self.gl, get_attrib_location, &program, name);
        self.check_error(&format!("get_attrib_location {}", name));
        if loc == -1 {
            None
        } else {
            Some(loc as u32)
        }
    }

    pub fn create_buffer(&self) -> WebGLBuffer {
        let val = gl_call!(&self.gl, create_buffer).unwrap();
        WebGLBuffer(self.add(val.into()))
    }

    pub fn bind_buffer(&self, kind: BufferKind, buffer: &WebGLBuffer) {
        let buffer: web_sys::WebGlBuffer = self.get(buffer.0).unwrap().into();
        gl_call!(&self.gl, bind_buffer, kind as u32, Some(&buffer));
    }

    pub fn buffer_data(&self, kind: BufferKind, data: &[u8], draw: DrawMode) {
        gl_call!(
            &self.gl,
            buffer_data_with_u8_array,
            kind as u32,
            data,
            draw as u32
        );
    }

    pub fn create_vertex_array(&self) -> WebGLVertexArray {
        let val = match &self.gl {
            WebContext::Gl2(gl) => gl.create_vertex_array().unwrap(),
            WebContext::Gl(_gl) => JsValue::from_f64(0.0).into(), // not supported on webgl
        };
        WebGLVertexArray(self.add(val.into()))
    }

    pub fn bind_vertex_array(&self, vao: &WebGLVertexArray) {
        let vao: web_sys::WebGlVertexArrayObject = self.get(vao.0).unwrap().into();
        match &self.gl {
            WebContext::Gl2(gl) => gl.bind_vertex_array(Some(&vao)),
            WebContext::Gl(_) => (), // not supported on webgl
        }
    }

    pub fn vertex_attrib_pointer(
        &self,
        location: u32,
        size: AttributeSize,
        kind: DataType,
        normalized: bool,
        stride: u32,
        offset: u32,
    ) {
        gl_call!(
            &self.gl,
            vertex_attrib_pointer_with_i32,
            location,
            size as i32,
            kind as u32,
            normalized,
            stride as i32,
            offset as i32
        );
    }

    pub fn enable_vertex_attrib_array(&self, location: u32) {
        gl_call!(&self.gl, enable_vertex_attrib_array, location);
    }

    pub fn draw_arrays(&self, mode: Primitives, count: usize) {
        gl_call!(&self.gl, draw_arrays, mode as u32, 0, count as i32);
    }

    fn check_error(&self, msg: &str) {
        let code = gl_call!(&self.gl, get_error);
        if code != web_sys::WebGl2RenderingContext::NO_ERROR {
            print(&format!(
                "ERROR {} {}",
                msg,
                match code {
                    web_sys::WebGl2RenderingContext::INVALID_ENUM => "invalid enum",
                    web_sys::WebGl2RenderingContext::INVALID_OPERATION => "invalid operation",
                    web_sys::WebGl2RenderingContext::INVALID_VALUE => "invalid value",
                    web_sys::WebGl2RenderingContext::OUT_OF_MEMORY => "out of memory",
                    web_sys::WebGl2RenderingContext::INVALID_FRAMEBUFFER_OPERATION =>
                        "invalid framebuffer operation",
                    web_sys::WebGl2RenderingContext::CONTEXT_LOST_WEBGL => "context lost webgl",
                    _ => "unknown error",
                },
            ));
        }
    }

    pub fn create_shader(&self, kind: ShaderKind) -> WebGLShader {
        if let Some(val) = gl_call!(&self.gl, create_shader, kind as u32) {
            let id = self.add(val.into());
            return WebGLShader(id);
        }
        self.check_error("create_shader");
        unreachable!();
    }

    pub fn shader_source(&self, shader: &WebGLShader, code: &str) {
        let shader: web_sys::WebGlShader = self.get(shader.0).unwrap().into();
        gl_call!(&self.gl, shader_source, &shader, code);
        self.log(&format!("shader source:\n{}", code));
    }

    pub fn create_program(&self) -> WebGLProgram {
        let val = gl_call!(&self.gl, create_program).unwrap();
        WebGLProgram(self.add(val.into()))
    }

    pub fn link_program(&self, program: &WebGLProgram) {
        let program: web_sys::WebGlProgram = self.get(program.0).unwrap().into();
        gl_call!(&self.gl, link_program, &program);
        let result = gl_call!(
            &self.gl,
            get_program_parameter,
            &program,
            web_sys::WebGl2RenderingContext::LINK_STATUS
        );
        if !result {
            print("ERROR while linking program :");
            print(&format!(
                "{}",
                gl_call!(&self.gl, get_program_info_log, &program).unwrap()
            ));
        }
    }

    pub fn attach_shader(&self, program: &WebGLProgram, shader: &WebGLShader) {
        let program: web_sys::WebGlProgram = self.get(program.0).unwrap().into();
        let shader: web_sys::WebGlShader = self.get(shader.0).unwrap().into();
        gl_call!(&self.gl, attach_shader, &program, &shader);
    }

    pub fn delete_buffer(&self, buffer: &WebGLBuffer) {
        let id = buffer.0;
        let buffer: web_sys::WebGlBuffer = self.get(id).unwrap().into();
        gl_call!(&self.gl, delete_buffer, Some(&buffer));
        self.remove(id);
    }

    pub fn unbind_buffer(&self, kind: BufferKind) {
        gl_call!(&self.gl, bind_buffer, kind as u32, None);
    }

    pub fn bind_attrib_location(&self, program: &WebGLProgram, name: &str, loc: u32) {
        let program: web_sys::WebGlProgram = self.get(program.0).unwrap().into();
        gl_call!(&self.gl, bind_attrib_location, &program, loc, name);
    }

    pub fn get_uniform_location(
        &self,
        program: &WebGLProgram,
        name: &str,
    ) -> Option<WebGLUniformLocation> {
        let program: web_sys::WebGlProgram = self.get(program.0).unwrap().into();
        let val = gl_call!(&self.gl, get_uniform_location, &program, name);
        val.map(|v| WebGLUniformLocation {
            reference: self.add(v.into()),
            name: name.to_string(),
        })
    }

    pub fn enable(&self, flag: i32) {
        gl_call!(&self.gl, enable, flag as u32);
    }

    pub fn disable(&self, flag: i32) {
        gl_call!(&self.gl, disable, flag as u32);
    }

    pub fn cull_face(&self, flag: Culling) {
        gl_call!(&self.gl, cull_face, flag as u32);
    }

    pub fn depth_mask(&self, is_on: bool) {
        gl_call!(&self.gl, depth_mask, is_on);
    }

    pub fn depth_func(&self, d: DepthTest) {
        gl_call!(&self.gl, depth_func, d as u32);
    }

    pub fn clear_depth(&self, value: f32) {
        gl_call!(&self.gl, clear_depth, value);
    }

    pub fn viewport(&self, x: i32, y: i32, width: u32, height: u32) {
        gl_call!(&self.gl, viewport, x, y, width as i32, height as i32);
    }

    pub fn draw_elements(&self, mode: Primitives, count: usize, kind: DataType, offset: u32) {
        gl_call!(
            &self.gl,
            draw_elements_with_i32,
            mode as u32,
            count as i32,
            kind as u32,
            offset as i32
        );
    }

    pub fn generate_mipmap(&self) {
        gl_call!(
            &self.gl,
            generate_mipmap,
            web_sys::WebGl2RenderingContext::TEXTURE_2D
        );
    }

    pub fn generate_mipmap_cube(&self) {
        gl_call!(
            &self.gl,
            generate_mipmap,
            web_sys::WebGl2RenderingContext::TEXTURE_CUBE_MAP
        );
    }

    pub fn create_texture(&self) -> WebGLTexture {
        let val = gl_call!(&self.gl, create_texture);
        WebGLTexture(self.add(val.into()))
    }

    pub fn delete_texture(&self, texture: &WebGLTexture) {
        let id = texture.0;
        let texture: web_sys::WebGlTexture = self.get(id).unwrap().into();
        gl_call!(&self.gl, delete_texture, Some(&texture));
        self.remove(id);
    }

    pub fn active_texture(&self, active: u32) {
        gl_call!(
            &self.gl,
            active_texture,
            web_sys::WebGl2RenderingContext::TEXTURE0 + active
        );
    }

    pub fn bind_texture(&self, texture: &WebGLTexture) {
        let texture: web_sys::WebGlTexture = self.get(texture.0).unwrap().into();
        gl_call!(
            &self.gl,
            bind_texture,
            TextureKind::Texture2d as u32,
            Some(&texture)
        );
    }

    pub fn unbind_texture(&self) {
        gl_call!(&self.gl, bind_texture, TextureKind::Texture2d as u32, None);
    }

    pub fn bind_texture_cube(&self, texture: &WebGLTexture) {
        let texture: web_sys::WebGlTexture = self.get(texture.0).unwrap().into();
        gl_call!(
            &self.gl,
            bind_texture,
            TextureKind::TextureCubeMap as u32,
            Some(&texture)
        );
    }

    pub fn unbind_texture_cube(&self) {
        gl_call!(
            &self.gl,
            bind_texture,
            TextureKind::TextureCubeMap as u32,
            None
        );
    }

    pub fn blend_equation(&self, eq: BlendEquation) {
        gl_call!(&self.gl, blend_equation, eq as u32);
    }

    pub fn blend_func(&self, sfactor: BlendMode, dfactor: BlendMode) {
        gl_call!(&self.gl, blend_func, sfactor as u32, dfactor as u32);
    }

    pub fn blend_color(&self, r: f32, g: f32, b: f32, a: f32) {
        gl_call!(&self.gl, blend_color, r, g, b, a);
    }

    pub fn create_framebuffer(&self) -> WebGLFrameBuffer {
        let val = gl_call!(&self.gl, create_framebuffer).unwrap();
        WebGLFrameBuffer(self.add(val.into()))
    }

    pub fn delete_framebuffer(&self, fb: &WebGLFrameBuffer) {
        let id = fb.0;
        let fb: web_sys::WebGlFramebuffer = self.get(id).unwrap().into();
        gl_call!(&self.gl, delete_framebuffer, Some(&fb));
        self.remove(id);
    }

    pub fn bind_framebuffer(&self, buffer: Buffers, fb: &WebGLFrameBuffer) {
        let fb: web_sys::WebGlFramebuffer = self.get(fb.0).unwrap().into();
        gl_call!(&self.gl, bind_framebuffer, buffer as u32, Some(&fb));
    }

    pub fn framebuffer_texture2d(
        &self,
        target: Buffers,
        attachment: Buffers,
        textarget: TextureBindPoint,
        texture: &WebGLTexture,
        level: i32,
    ) {
        let texture: web_sys::WebGlTexture = self.get(texture.0).unwrap().into();
        gl_call!(
            &self.gl,
            framebuffer_texture_2d,
            target as u32,
            attachment as u32,
            textarget as u32,
            Some(&texture),
            level
        );
    }

    pub fn unbind_framebuffer(&self, buffer: Buffers) {
        gl_call!(&self.gl, bind_framebuffer, buffer as u32, None);
    }

    pub fn tex_parameteri(&self, kind: TextureKind, pname: TextureParameter, param: i32) {
        // skip not supported flag in for webgl 1 context
        if !self.is_webgl2 {
            if let TextureParameter::TextureWrapR = pname {
                return;
            }
        }
        gl_call!(&self.gl, tex_parameteri, kind as u32, pname as u32, param);
    }

    pub fn tex_parameterfv(&self, kind: TextureKind, pname: TextureParameter, param: f32) {
        gl_call!(&self.gl, tex_parameterf, kind as u32, pname as u32, param);
    }

    pub fn draw_buffer(&self, buffers: &[ColorBuffer]) {
        match &self.gl {
            WebContext::Gl2(gl) => {
                let color_enums: Array = buffers
                    .iter()
                    .map(|c| JsValue::from(*c as i32))
                    .collect::<Array>();
                gl.draw_buffers(&color_enums);
            }
            WebContext::Gl(_) => (), // not supported
        }
    }

    pub fn uniform_matrix_3fv(&self, location: &WebGLUniformLocation, value: &[[f32; 3]; 3]) {
        use std::mem;
        let array = unsafe { mem::transmute::<&[[f32; 3]; 3], &[f32; 9]>(value) as &[f32] };
        let location: web_sys::WebGlUniformLocation = self.get(location.reference).unwrap().into();
        gl_call!(
            &self.gl,
            uniform_matrix3fv_with_f32_array,
            Some(&location),
            false,
            &array
        );
    }

    pub fn uniform_matrix_2fv(&self, location: &WebGLUniformLocation, value: &[[f32; 2]; 2]) {
        use std::mem;
        let array = unsafe { mem::transmute::<&[[f32; 2]; 2], &[f32; 4]>(value) as &[f32] };
        let location: web_sys::WebGlUniformLocation = self.get(location.reference).unwrap().into();
        gl_call!(
            &self.gl,
            uniform_matrix2fv_with_f32_array,
            Some(&location),
            false,
            &array
        );
    }

    pub fn uniform_1i(&self, location: &WebGLUniformLocation, value: i32) {
        let location: web_sys::WebGlUniformLocation = self.get(location.reference).unwrap().into();
        gl_call!(&self.gl, uniform1i, Some(&location), value);
    }

    pub fn uniform_1f(&self, location: &WebGLUniformLocation, value: f32) {
        let location: web_sys::WebGlUniformLocation = self.get(location.reference).unwrap().into();
        gl_call!(&self.gl, uniform1f, Some(&location), value);
    }

    pub fn uniform_2f(&self, location: &WebGLUniformLocation, value: (f32, f32)) {
        let location: web_sys::WebGlUniformLocation = self.get(location.reference).unwrap().into();
        gl_call!(&self.gl, uniform2f, Some(&location), value.0, value.1);
    }

    pub fn uniform_3f(&self, location: &WebGLUniformLocation, value: (f32, f32, f32)) {
        let location: web_sys::WebGlUniformLocation = self.get(location.reference).unwrap().into();
        gl_call!(
            &self.gl,
            uniform3f,
            Some(&location),
            value.0,
            value.1,
            value.2
        );
    }

    pub fn uniform_4f(&self, location: &WebGLUniformLocation, value: (f32, f32, f32, f32)) {
        let location: web_sys::WebGlUniformLocation = self.get(location.reference).unwrap().into();
        gl_call!(
            &self.gl,
            uniform4f,
            Some(&location),
            value.0,
            value.1,
            value.2,
            value.3
        );
    }

    pub fn uniform_matrix_4fv(&self, location: &WebGLUniformLocation, value: &[[f32; 4]; 4]) {
        use std::mem;
        let array = unsafe { mem::transmute::<&[[f32; 4]; 4], &[f32; 16]>(value) as &[f32] };
        let location: web_sys::WebGlUniformLocation = self.get(location.reference).unwrap().into();
        gl_call!(
            &self.gl,
            uniform_matrix4fv_with_f32_array,
            Some(&location),
            false,
            array
        );
    }

    pub fn delete_vertex_array(&self, vao: &WebGLVertexArray) {
        let id = vao.0;
        match &self.gl {
            WebContext::Gl2(gl) => {
                let vao: web_sys::WebGlVertexArrayObject = self.get(id).unwrap().into();
                gl.delete_vertex_array(Some(&vao));
            }
            WebContext::Gl(_) => (), // unsupported
        }
        self.remove(id);
    }

    pub fn unbind_vertex_array(&self, _vao: &WebGLVertexArray) {
        match &self.gl {
            WebContext::Gl2(gl) => {
                gl.bind_vertex_array(None);
            }
            WebContext::Gl(_) => (), // unsupported
        }
    }

    pub fn get_program_parameter(&self, program: &WebGLProgram, pname: ShaderParameter) -> i32 {
        let program: web_sys::WebGlProgram = self.get(program.0).unwrap().into();
        let val = gl_call!(&self.gl, get_program_parameter, &program, pname as u32);
        val.as_f64().unwrap() as i32
    }

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
        if pixels.len() > 0 {
            gl_call!(
                &self.gl,
                tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array,
                target as u32,
                level as i32,
                format as i32,
                width as i32,
                height as i32,
                0,
                format as u32,
                kind as u32,
                Some(pixels)
            )
            .unwrap();
        } else {
            // TODO: It is a strange bug !!!
            // According https://developer.mozilla.org/en-US/docs/Web/API/WebGLRenderingContext/texImage2D
            // the format arg should be equal to internal format arg
            // however, only DEPTH_COMPONENT16 works but not DEPTH_COMPONENT

            let internal_format = match format {
                PixelFormat::DepthComponent => web_sys::WebGl2RenderingContext::DEPTH_COMPONENT16,
                _ => format as u32,
            };
            gl_call!(
                &self.gl,
                tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array,
                target as u32,
                level as i32,
                internal_format as i32,
                width as i32,
                height as i32,
                0,
                format as u32,
                kind as u32,
                None
            )
            .unwrap();
        }
    }

    pub fn pixel_storei(&self, storage: PixelStorageMode, value: i32) {
        gl_call!(&self.gl, pixel_storei, storage as u32, value);
    }

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
        gl_call!(
            &self.gl,
            read_pixels_with_opt_u8_array,
            x as i32,
            y as i32,
            width as i32,
            height as i32,
            format as u32,
            kind as u32,
            Some(data)
        )
        .unwrap();
    }

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
        gl_call!(
            &self.gl,
            tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array,
            target as u32,
            level as i32,
            xoffset as i32,
            yoffset as i32,
            width as i32,
            height as i32,
            format as u32,
            kind as u32,
            Some(pixels)
        )
        .unwrap();
    }

    pub fn compressed_tex_image2d(
        &self,
        target: TextureBindPoint,
        level: u8,
        compression: TextureCompression,
        width: u16,
        height: u16,
        data: &[u8],
    ) {
        // for some reason this needs to be called otherwise invalid format error, extension initialization?
        let _ = self.get_extension("WEBGL_compressed_texture_s3tc")
            || self.get_extension("MOZ_WEBGL_compressed_texture_s3tc")
            || self.get_extension("WEBKIT_WEBGL_compressed_texture_s3tc");
        gl_call!(
            &self.gl,
            compressed_tex_image_2d_with_u8_array,
            target as u32,
            level as i32,
            compression as u32,
            width as i32,
            height as i32,
            0,
            data
        );
    }
    /*
       // pub fn get_active_uniform(&self, program: &WebGLProgram, location: u32) -> WebGLActiveInfo {
       //     let res = js! {
       //         var h = Module.gl.get(@{program.deref()});
       //         var ctx = Module.gl.get(@{self.reference});

       //         return ctx.getActiveUniform(h.prog,@{location})
       //     };

       //     let name = js! { return @{&res}.name };
       //     let size = js!{ return @{&res}.size };
       //     let kind = js!{ return @{&res}.type };
       //     let k: u32 = kind.try_into().unwrap();
       //     use std::mem;
       //     WebGLActiveInfo::new(
       //         name.into_string().unwrap(),
       //         size.try_into().unwrap(),
       //         unsafe { mem::transmute::<u16, UniformType>(k as _) },
       //         res.into_reference().unwrap(),
       //     )
       // }

       // pub fn get_active_attrib(&self, program: &WebGLProgram, location: u32) -> WebGLActiveInfo {
       //     let res = js! {
       //         var h = Module.gl.programs[@{program.deref()}];
       //         return @{self.reference}.getActiveAttrib(h.prog,@{location})
       //     };
       //     let name = js! { return @{&res}.name };
       //     let size = js!{ return @{&res}.size };
       //     let kind = js!{ return @{&res}.type };
       //     let k: u32 = kind.try_into().unwrap();
       //     use std::mem;
       //     WebGLActiveInfo::new(
       //         name.into_string().unwrap(),
       //         size.try_into().unwrap(),
       //         unsafe { mem::transmute::<u16, UniformType>(k as _) },
       //         res.into_reference().unwrap(),
       //     )
       // }

    */
}
