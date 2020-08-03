use wasm_bindgen::prelude::*;

mod utils;
mod gui;
use gui::CCGui;
mod input;
mod player_ui;
mod resources;
mod audio_ended_event;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const UPS: f64 = 60.0;
const LOCKSTEP: bool = true;

#[wasm_bindgen]
pub fn main() {
    utils::set_panic_hook();
    wasm_bindgen_futures::spawn_local(async {
        let mut gui = CCGui::new().await;

        let mut prev_time = utils::window().performance().unwrap().now();
        let mut frametimes = [1000.0 / 60.0; 10];
        let mut alpha = 0.0;
        loop {
            let now = webutil::global::animation_frame().await;
            frametimes[0] = now - prev_time;
            frametimes.rotate_left(1);
            prev_time = now;
    
            let frametime = frametimes.iter().sum::<f64>() / 10.0;
            let frametime = frametime / 1000.0;
    
            let (lockstep_low, lockstep_high) = lockstep_tolerance(UPS);
    
            let high_framerate = frametime < lockstep_low;
            let low_framerate = frametime > lockstep_high;

            if low_framerate || high_framerate || !LOCKSTEP {
                alpha += frametime * UPS;
            } else {
                alpha = 2.0;
            }

            let mut updates: u32 = 0;
            while alpha > 1.0 {
                updates += 1;
                if updates as f64 > UPS / 12.0 {
                    alpha = alpha.min(2.0);
                }
                alpha -= 1.0;
                gui.update().await;
            }
            
            gui.render(frametime);
        }
    });
}

fn lockstep_tolerance(ups: f64) -> (f64, f64) {
    let ms_lower_bound = 1.0/ups - 0.001;
    let hz_lower_bound = 1.0/(ups + 2.0);

    let ms_upper_bound = 1.0/ups + 0.001;
    let hz_upper_bound = 1.0/(ups - 2.0);

    (ms_lower_bound.max(hz_lower_bound), ms_upper_bound.min(hz_upper_bound))
}
