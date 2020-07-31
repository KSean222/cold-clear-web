use std::rc::Rc;
use std::cell::RefCell;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

mod utils;
mod gui;
use gui::CCGui;
mod input;
mod player_ui;
mod resources;
use resources::Resources;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub const UPS: f64 = 60.0;

#[wasm_bindgen]
pub fn main() {
    utils::set_panic_hook();
    wasm_bindgen_futures::spawn_local(async {
        let resources = Resources::load().await;
        let mut gui = CCGui::new().await;
        fn request_animation_frame(func: &Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>) {
            let func = func.borrow();
            let func = func
                .as_ref()
                .unwrap()
                .as_ref()
                .unchecked_ref();
            utils::window().request_animation_frame(func).unwrap();
        }
        let step = Rc::new(RefCell::new(None));
        *step.borrow_mut() = Some(Closure::wrap(Box::new({
            let step = step.clone();
            
            fn lockstep_tolerance(ups: f64) -> (f64, f64) {
                let ms_lower_bound = 1.0/ups - 0.001;
                let hz_lower_bound = 1.0/(ups + 2.0);
            
                let ms_upper_bound = 1.0/ups + 0.001;
                let hz_upper_bound = 1.0/(ups - 2.0);
            
                (ms_lower_bound.max(hz_lower_bound), ms_upper_bound.min(hz_upper_bound))
            }
            const LOCKSTEP: bool = true;
            let mut prev_time = utils::window().performance().unwrap().now();
            let mut frametimes = [1000.0 / 60.0; 10];
            let mut alpha = 0.0;
            let mut paused = false;
            let mut low_framerate = false;
            move |now: f64| {
                // wasm_bindgen_futures::spawn_local(async move {
                    frametimes[0] = now - prev_time;
                    frametimes.rotate_left(1);
                    prev_time = now;
            
                    let frametime = frametimes.iter().sum::<f64>() / 10.0;
                    let frametime = frametime / 1000.0;
            
                    let (lockstep_low, lockstep_high) = lockstep_tolerance(UPS);
            
                    let high_framerate = frametime < lockstep_low;
                    low_framerate = frametime > lockstep_high;
        
                    if low_framerate || high_framerate || !LOCKSTEP {
                        alpha += frametime * UPS;
                    } else {
                        alpha = 2.0;
                    }
        
                    let mut updates: u32 = 0;
                    while alpha > 1.0 && !paused {
                        updates += 1;
                        if updates as f64 > UPS / 12.0 {
                            alpha = alpha.min(2.0);
                        }
                        alpha -= 1.0;
                        gui.update();//.await;
                    }
                    
                    gui.render(&resources, frametime);
                    request_animation_frame(&step);
                // });
            }
        }) as Box<dyn FnMut(f64)>));
        request_animation_frame(&step);
    });
}

