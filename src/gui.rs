use wasm_bindgen::JsCast;

use crate::player_ui::PlayerUi;
use crate::utils;
use crate::resources::Resources;

use battle::{Battle, GameConfig};

const START_COUNTDOWN: u32 = 3;
fn random_seed() -> [u8; 16] {
    let mut seed = [0; 16];
    for n in &mut seed {
        *n = (js_sys::Math::random() * 256.0) as u8;
    }
    seed
}

pub struct CCGui {
    p1_ui: PlayerUi,
    p2_ui: PlayerUi,
    fps_text: web_sys::HtmlElement,
    elapsed: u32,
    countdown_text: web_sys::HtmlElement,
    timer_text: web_sys::HtmlElement,
    battle: Battle
}

impl CCGui {
    pub fn new() -> Self {
        let document = utils::document();
        let body = utils::body();

        let p1_ui = PlayerUi::new();
        let p1_element = p1_ui.element();
        p1_element.set_id("player-one");
        body.append_child(p1_element).unwrap();
        
        let p2_ui = PlayerUi::new();
        let p2_element = p2_ui.element();
        p2_element.set_id("player-two");
        body.append_child(p2_element).unwrap();

        let fps_text: web_sys::HtmlElement = document
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        fps_text.set_id("fps-text");
        body.append_child(&fps_text).unwrap();

        let countdown_text: web_sys::HtmlElement = document
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        countdown_text.set_id("countdown-text");
        body.append_child(&countdown_text).unwrap();

        let timer_text: web_sys::HtmlElement = document
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        timer_text.set_id("timer-text");
        body.append_child(&timer_text).unwrap();
        
        Self {
            p1_ui,
            p2_ui,
            fps_text,
            countdown_text,
            timer_text,
            elapsed: 0,
            battle: Battle::new(
                GameConfig::default(), 
                GameConfig::default(), 
                random_seed(),
                random_seed(),
                random_seed()
            )
        }
    }
    pub fn update(&mut self) {
        self.elapsed += 1;
    }
    pub fn render(&mut self, resources: &Resources, smooth_delta: f64) {
        self.fps_text.set_inner_text(&format!("FPS: {:.0}", 1.0 / smooth_delta));
        let elapsed_seconds = (self.elapsed / crate::UPS as u32);
        if elapsed_seconds < START_COUNTDOWN {
            self.countdown_text.set_inner_text(&format!("{}", START_COUNTDOWN - elapsed_seconds));
        } else {
            self.countdown_text.set_inner_text("");
        }
        let timer_seconds = elapsed_seconds.max(START_COUNTDOWN) - START_COUNTDOWN;
        self.timer_text.set_inner_text(&format!("{}:{:02}", timer_seconds / 60, timer_seconds % 60));
        self.p1_ui.render(resources);
        self.p2_ui.render(resources);
    }
}