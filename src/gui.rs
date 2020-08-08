use wasm_bindgen::JsCast;

use crate::player_ui::PlayerUi;
use crate::utils;
use crate::resources::Resources;
use crate::input::*;
use crate::options::*;

use battle::Battle;

const START_COUNTDOWN: u32 = 3;
const RESET_COUNTDOWN: u32 = 5;
fn random_seed() -> [u8; 16] {
    let mut seed = [0; 16];
    for n in &mut seed {
        *n = (js_sys::Math::random() * 256.0) as u8;
    }
    seed
}

pub struct CCGui {
    resources: Resources,
    options: Options,
    p1_input: Box<dyn InputSource>,
    p2_input: Box<dyn InputSource>,
    p1_ui: PlayerUi,
    p2_ui: PlayerUi,
    p1_wins: u32,
    p2_wins: u32,
    fps_text: web_sys::HtmlElement,
    countdown: u32,
    countdown_text: web_sys::HtmlElement,
    timer_text: web_sys::HtmlElement,
    win_count_text: web_sys::HtmlElement,
    game_over: bool,
    reset_countdown: u32,
    battle: Battle
}

impl CCGui {
    pub async fn new() -> Self {
        let resources = Resources::load().await.unwrap();

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
        
        let win_count_text: web_sys::HtmlElement = document
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        win_count_text.set_id("win-counter-text");
        middle_text.append_child(&win_count_text).unwrap();
        
        let options = Options::read();
        let battle = Battle::new(
            options.p1.game.clone(), 
            options.p2.game.clone(), 
            random_seed(),
            random_seed(),
            random_seed()
        );
        let (p1_input, p1_name) =
            options.p1.to_player(battle.player_1.board.to_compressed()).await;
        let (p2_input, p2_name) =
            options.p2.to_player(battle.player_2.board.to_compressed()).await;

        Self {
            resources,
            options,
            p1_ui,
            p2_ui,
            p1_input,
            p2_input,
            p1_wins: 0,
            p2_wins: 0,
            fps_text,
            countdown_text,
            timer_text,
            win_count_text,
            countdown: START_COUNTDOWN * crate::UPS as u32,
            game_over: false,
            reset_countdown: 0,
            battle
        }
    }
    pub async fn update(&mut self) {
        if self.countdown > 0 {
            self.countdown -= 1;
        } else {
            let update = self.battle.update(self.p1_input.controller(), self.p2_input.controller());
            let p1_info = self.p1_input.update(&self.battle.player_1.board, &update.player_1.events, update.player_1.garbage_queue);
            let p2_info = self.p2_input.update(&self.battle.player_2.board, &update.player_2.events, update.player_2.garbage_queue);
            self.p1_ui.update(&self.resources, &update.player_1.events, p1_info);
            self.p2_ui.update(&self.resources, &update.player_2.events, p2_info);
            if self.game_over {
                if self.reset_countdown > 0 {
                    self.reset_countdown -= 1;
                } else {
                    self.game_over = false;
                    self.countdown = START_COUNTDOWN * crate::UPS as u32;
                    
                    self.p1_ui.reset();
                    self.p2_ui.reset();
                    self.options = Options::read();
                    self.battle = Battle::new(
                        self.options.p1.game.clone(), 
                        self.options.p2.game.clone(), 
                        random_seed(),
                        random_seed(),
                        random_seed()
                    );
                    let (p1_input, p1_name) =
                        self.options.p1.to_player(self.battle.player_1.board.to_compressed()).await;
                    let (p2_input, p2_name) =
                        self.options.p2.to_player(self.battle.player_2.board.to_compressed()).await;
                    self.p1_input = p1_input;
                    self.p2_input = p2_input;
                }
            } else {
                for event in &update.player_1.events {
                    if let battle::Event::GameOver = event {
                        self.p2_wins += 1;
                        self.game_over = true;
                    }
                }
                for event in &update.player_2.events {
                    if let battle::Event::GameOver = event {
                        self.p1_wins += 1;
                        self.game_over = true;
                    }
                }
                if self.game_over {
                    self.reset_countdown = RESET_COUNTDOWN * crate::UPS as u32;
                }
            }
        }
    }
    pub fn render(&self, smooth_delta: f64) {
        self.fps_text.set_inner_text(&format!("FPS: {:.0}", 1.0 / smooth_delta));
        if self.countdown > 0 {
            self.countdown_text.set_inner_text(&format!("{}", self.countdown / crate::UPS as u32 + 1));
        } else {
            self.countdown_text.set_inner_text("");
        }
        let elapsed_seconds = self.battle.time / 60;
        self.timer_text.set_inner_text(&format!("{}:{:02}", elapsed_seconds / 60, elapsed_seconds % 60));
        self.win_count_text.set_inner_text(&format!("{} - {}", self.p1_wins, self.p2_wins));
        self.p1_ui.render(&self.resources, &self.battle.player_1, &self.options.p1);
        self.p2_ui.render(&self.resources, &self.battle.player_2, &self.options.p2);
    }
}