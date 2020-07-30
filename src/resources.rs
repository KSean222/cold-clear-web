use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use js_sys::*;

pub struct Resources {
    pub skin: web_sys::HtmlImageElement,
}

async fn image(src: &str) -> Result<web_sys::HtmlImageElement, JsValue> {
    let img = web_sys::HtmlImageElement::new()?;
    JsFuture::from(Promise::new(&mut |res, rej| {
        img.set_src(src);
        img.set_onload(Some(&res));
        img.set_onerror(Some(&rej));
    })).await.map(|_| img)
}

impl Resources {
    pub async fn load() -> Self {
        Self {
            skin: image("./res/skin.png").await.unwrap()
        }
    }
}
