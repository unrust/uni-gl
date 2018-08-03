extern crate uni_app;
extern crate uni_gl;

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
    });
    // retrieve the opengl context
    let gl = uni_gl::WebGLRenderingContext::new(app.canvas());
    // start game loop
    app.run(move |_app: &mut uni_app::App| {
        // do some openGL stuff
        gl.clear_color(0.0, 0.0, 1.0, 1.0);
        gl.clear(uni_gl::BufferBit::Color);
    });
}
