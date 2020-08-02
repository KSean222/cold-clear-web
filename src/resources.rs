use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use js_sys::*;

use crate::utils;

use libtetris::*;
use enum_map::EnumMap;

pub struct Resources {
    pub skin: web_sys::HtmlImageElement,
    pub pieces: EnumMap<Piece, web_sys::HtmlCanvasElement>,
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
        let skin = image("./res/sprites/skin.png").await?;
        let hard_drop_sfx = audio(&audio_context, "./res/sounds/hard-drop.ogg").await?;
        let line_clear_sfx = audio(&audio_context, "./res/sounds/line-clear.ogg").await?;
        let move_sfx = audio(&audio_context, "./res/sounds/move.ogg").await?;

        let cell_size = skin.height() / 2;
        let pieces = (|piece: Piece| {
            let (canvas, context) = utils::new_canvas();

            let cells = PieceState(piece, RotationState::North).cells();
            let min_x = cells.iter().map(|c| c.0).min().unwrap();
            let max_x = cells.iter().map(|c| c.0).max().unwrap();
            let min_y = cells.iter().map(|c| -c.1).min().unwrap();
            let max_y = cells.iter().map(|c| -c.1).max().unwrap();
            let width = max_x - min_x + 1;
            let height = max_y - min_y + 1;
            canvas.set_width(width as u32 * cell_size);
            canvas.set_height(height as u32 * cell_size);

            let (src_x, src_y) = Resources::cell_pos(piece.color(), false);
            for &(mut x, mut y) in &cells {
                y = -y;
                x -= min_x;
                y -= min_y;
                context
                    .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                        &skin,
                        (src_x as u32 * cell_size) as f64,
                        (src_y as u32 * cell_size) as f64,
                        cell_size as f64,
                        cell_size as f64,
                        (x as u32 * cell_size) as f64,
                        (y as u32 * cell_size) as f64,
                        cell_size as f64,
                        cell_size as f64
                    )
                    .unwrap();
            }
            canvas
        }).into();

        Ok(Self {
            skin,
            pieces,
            hard_drop_sfx,
            line_clear_sfx,
            move_sfx,
            audio_context,
        })
    }
    pub fn cell_pos(cell: CellColor, is_ghost: bool) -> (u32, u32) {
        let x = match cell {
            CellColor::Unclearable => 1,
            CellColor::Garbage => 2,
            CellColor::Z => 3,
            CellColor::L => 4,
            CellColor::O => 5,
            CellColor::S => 6,
            CellColor::I => 7,
            CellColor::J => 8,
            CellColor::T => 9,
            _ => 0
        };
        let y = if is_ghost { 1 } else { 0 };
        (x, y)
    }
}
