use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn window() -> web_sys::Window {
    web_sys::window().unwrap()
}

pub fn document() -> web_sys::Document {
    window().document().unwrap()
}

pub fn body() -> web_sys::HtmlElement {
    document().body().unwrap()
}

pub fn new_canvas() -> (web_sys::HtmlCanvasElement, web_sys::CanvasRenderingContext2d) {
    let canvas: web_sys::HtmlCanvasElement = document()
        .create_element("canvas")
        .unwrap()
        .dyn_into()
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap();
    (canvas, context)
}