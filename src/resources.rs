use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use js_sys::*;

use crate::utils;

pub struct Resources {
    pub skin: web_sys::HtmlImageElement,
    pub hard_drop_sfx: web_sys::AudioBuffer,
    pub line_clear_sfx: web_sys::AudioBuffer,
    pub move_sfx: web_sys::AudioBuffer,
    pub audio_context: web_sys::AudioContext
}

async fn image(src: &str) -> Result<web_sys::HtmlImageElement, JsValue> {
    let image = web_sys::HtmlImageElement::new()?;
    JsFuture::from(Promise::new(&mut |res, rej| {
        image.set_src(src);
        image.set_onload(Some(&res));
        image.set_onerror(Some(&rej));
    })).await.map(|_| image)
}

async fn audio(context: &web_sys::AudioContext, src: &str) -> Result<web_sys::AudioBuffer, JsValue> {
    let response: web_sys::Response = JsFuture::from(utils::window().fetch_with_str(src))
        .await?
        .dyn_into()
        .unwrap();
    let buffer: js_sys::ArrayBuffer = JsFuture::from(response.array_buffer()?)
        .await?
        .dyn_into()
        .unwrap();
    let buffer = JsFuture::from(context.decode_audio_data(&buffer)?)
        .await?
        .dyn_into()
        .unwrap();
    Ok(buffer)
}

impl Resources {
    pub async fn load() -> Result<Self, JsValue> {
        let audio_context = web_sys::AudioContext::new()?;
        Ok(Self {
            skin: image("./res/sprites/skin.png").await?,
            hard_drop_sfx: audio(&audio_context, "./res/sounds/hard-drop.ogg").await?,
            line_clear_sfx: audio(&audio_context, "./res/sounds/line-clear.ogg").await?,
            move_sfx: audio(&audio_context, "./res/sounds/move.ogg").await?,
            audio_context
        })
    }
}
