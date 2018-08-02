# unrust

[![Build Status](https://travis-ci.org/unrust/uni-gl.svg?branch=master)](https://travis-ci.org/unrust/uni-gl)

This library is a part of [Unrust](https://github.com/unrust/unrust), a pure rust native/wasm game engine.
This library provides a native/wasm compatibility layer for following components :
* OpenGL API

When used in conjonction with uni-app, on native target, it provides an OpenGL 3.2+ or OpenGLES 2.0+ Core Profile context.
On web target, it provides a WebGL 2.0 context where available, else a WebGL 1.0 context.

**This project is under heavily development, all api are very unstable until version 0.2**

## Usage 

```toml
[dependencies]
uni-app="0.1.*"
uni-gl="0.1.*"
```

```rust
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
```

## Build

### As web app (wasm32-unknown-unknown)


When targetting `wasm32-unknown-unknown`, stdweb currently requires Rust nightly.

```
cargo install --force cargo-web # installs web sub command
rustup override set nightly
rustup target install wasm32-unknown-unknown
cargo web start --example basic --release
```

### As desktop app (native-opengl)

Native compilation works with current stable Rust (1.28)

```
rustup override set stable
cargo run --example basic --release
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
