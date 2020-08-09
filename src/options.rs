use crate::utils;
use crate::input::*;

use serde::{Serialize, Deserialize};
use battle::GameConfig;
use cold_clear::evaluation::Evaluator;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Options {
    pub p1: PlayerConfig,
    pub p2: PlayerConfig
}

impl Default for Options {
    fn default() -> Self {
        let mut p2 = PlayerConfig::default();
        p2.is_bot = true;
        Options {
            p1: PlayerConfig::default(),
            p2
        }
    }
}

impl Options {
    pub fn read() -> Self {
        const OPTIONS: &'static str = "options";
        let local_storage = utils::window()
            .local_storage()
            .unwrap()
            .unwrap();
        if let Some(options) = local_storage.get_item(OPTIONS).unwrap() {
            serde_json::from_str(&options).unwrap()
        } else {
            let options = Self::default();
            local_storage.set_item(OPTIONS, &serde_json::to_string(&options).unwrap()).unwrap();
            options
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerConfig {
    pub controls: InputConfig<String>,
    pub game: GameConfig,
    pub bot_config: BotConfig,
    pub is_bot: bool,
}

impl PlayerConfig {
    pub async fn to_player(&self, board: libtetris::Board) -> (Box<dyn InputSource>, String) {
        if self.is_bot {
            let mut name = format!("Cold Clear ({})", self.bot_config.evaluator.name());
            if self.bot_config.speed_limit != 0 {
                name.push_str(
                    &format!(" ({:.1}%)", 100.0 / (self.bot_config.speed_limit + 1) as f32)
                );
            }
            (Box::new(BotInput::new(cold_clear::Interface::launch(
                board,
                self.bot_config.options,
                self.bot_config.evaluator.clone(),
            ).await, self.bot_config.speed_limit)), name)
        } else {
            (Box::new(KeyboardInput::new(self.controls.clone())), "Human".to_owned())
        }
    }
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            controls: KeyboardInput::default_config(),
            game: GameConfig::default(),
            bot_config: BotConfig::default(),
            is_bot: false
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BotConfig {
    pub evaluator: cold_clear::evaluation::Standard,
    pub options: cold_clear::Options,
    pub speed_limit: u32
}
