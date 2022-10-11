# Changelog

## [0.2.1] - 2022-10-11
### Fixed
- missing disable function on webgl

## [0.2.0] - 2022-10-10
### Changed
- upgrade to rust edition 2021
- replace unmaintained cargo-web/stdweb with wasm_bindgen/web-sys

## [0.1.3] - 2020-02-15
### Changed
- upgrade to gl 0.14
- upgrade to stdweb 0.4.20

## [0.1.2] - 2019-12-04
### Fixed
- fixed discrepancy between native and webgl version (don't clear the drawbuffer between frames)

## [0.1.1] - 2019-02-01
### Fixed
- replaced invocation of broken js_raw macro with slower but working js macro

## [0.1.0] - 2018-06-24
- initial release