use wasm_bindgen::JsCast;

use crate::player_ui::PlayerUi;
use crate::utils;
use crate::resources::Resources;
use crate::input::*;

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
    p1_input: Box<dyn InputSource>,
    p2_input: Box<dyn InputSource>,
    fps_text: web_sys::HtmlElement,
    countdown: u32,
    countdown_text: web_sys::HtmlElement,
    timer_text: web_sys::HtmlElement,
    battle: Battle
}

impl CCGui {
    pub async fn new() -> Self {
        let document = utils::document();
        let body = utils::body();

        let fps_text: web_sys::HtmlElement = document
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        fps_text.set_id("fps-text");
        body.append_child(&fps_text).unwrap();

        let p1_ui = PlayerUi::new();
        let p1_element = p1_ui.element();
        p1_element.set_id("player-one");
        body.append_child(p1_element).unwrap();

        let p2_ui = PlayerUi::new();
        let p2_element = p2_ui.element();
        p2_element.set_id("player-two");
        body.append_child(p2_element).unwrap();
        
        let middle_text: web_sys::HtmlElement = document
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        middle_text.set_id("middle-text");
        body.append_child(&middle_text).unwrap();

        let countdown_text: web_sys::HtmlElement = document
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        countdown_text.set_id("countdown-text");
        middle_text.append_child(&countdown_text).unwrap();

        let timer_text: web_sys::HtmlElement = document
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        timer_text.set_id("timer-text");
        middle_text.append_child(&timer_text).unwrap();

        let battle = Battle::new(
            GameConfig::default(), 
            GameConfig::default(), 
            random_seed(),
            random_seed(),
            random_seed()
        );
        
        let board = battle.player_1.board.to_compressed();
        let options = cold_clear::Options::default();
        let evaluator = cold_clear::evaluation::Standard::default();
        let bot = BotInput::new(cold_clear::Interface::launch(board, options, evaluator).await, 0);
        let p1_input = Box::new(bot) as Box<dyn InputSource>;

        let board = battle.player_2.board.to_compressed();
        let options = cold_clear::Options::default();
        let evaluator = cold_clear::evaluation::Standard::default();
        let bot = BotInput::new(cold_clear::Interface::launch(board, options, evaluator).await, 0);
        let p2_input = Box::new(bot) as Box<dyn InputSource>;

        Self {
            p1_ui,
            p2_ui,
            p1_input,
            p2_input,
            fps_text,
            countdown_text,
            timer_text,
            countdown: START_COUNTDOWN * crate::UPS as u32,
            battle
        }
    }
    pub async fn update(&mut self, resources: &Resources) {
        if self.countdown > 0 {
            self.countdown -= 1;
        } else {
            let update = self.battle.update(self.p1_input.controller(), self.p2_input.controller());
            self.p1_input.update(&self.battle.player_1.board, &update.player_1.events, update.player_1.garbage_queue);
            self.p2_input.update(&self.battle.player_2.board, &update.player_2.events, update.player_2.garbage_queue);
            self.p1_ui.update(resources, &update.player_1.events);
            self.p2_ui.update(resources, &update.player_2.events);
        }
    }
    pub fn render(&self, resources: &Resources, smooth_delta: f64) {
        self.fps_text.set_inner_text(&format!("FPS: {:.0}", 1.0 / smooth_delta));
        if self.countdown > 0 {
            self.countdown_text.set_inner_text(&format!("{}", self.countdown / crate::UPS as u32 + 1));
        } else {
            self.countdown_text.set_inner_text("");
        }
        let elapsed_seconds = self.battle.time / 60;
        self.timer_text.set_inner_text(&format!("{}:{:02}", elapsed_seconds / 60, elapsed_seconds % 60));
        self.p1_ui.render(resources, &self.battle.player_1);
        self.p2_ui.render(resources, &self.battle.player_2);
    }
}