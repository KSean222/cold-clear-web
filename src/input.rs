use std::collections::HashSet;
use std::rc::Rc;
use std::cell::RefCell;

use crate::utils;

use serde::{Serialize, Deserialize};
use libtetris::*;
use battle::{Event, PieceMoveExecutor};
use webutil::event::EventTargetExt;

pub trait InputSource {
    fn controller(&self) -> Controller;
    fn update(
        &mut self, _board: &Board<ColoredRow>, _events: &[Event], _incoming: u32
    ) -> Option<cold_clear::Info> {
        None
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputConfig<T: Clone> {
    pub left: T,
    pub right: T,
    pub rotate_right: T,
    pub rotate_left: T,
    pub soft_drop: T,
    pub hard_drop: T,
    pub hold: T
}

pub struct KeyboardInput {
    pub config: InputConfig<String>,
    _keyup_listener: webutil::event::ListenerHandle,
    _keydown_listener: webutil::event::ListenerHandle,
    keys: Rc<RefCell<HashSet<String>>>
}

impl KeyboardInput {
    pub fn new(config: InputConfig<String>) -> Self {
        let window = utils::window();
        let keys = Rc::new(RefCell::new(HashSet::new()));
        Self {
            config,
            _keydown_listener: window.add_event_listener({
                let keys = Rc::clone(&keys);
                move |event: webutil::event::KeyDown| {
                    let mut keys = keys.borrow_mut();
                    keys.insert(event.code());
                }
            }),
            _keyup_listener: window.add_event_listener({
                let keys = Rc::clone(&keys);
                move |event: webutil::event::KeyUp| {
                    let mut keys = keys.borrow_mut();
                    keys.remove(&event.code());
                }
            }),
            keys
        }
    }
    pub fn default_config() -> InputConfig<String> {
        InputConfig {
            left: "ArrowLeft".to_owned(),
            right: "ArrowRight".to_owned(),
            rotate_left: "KeyZ".to_owned(),
            rotate_right: "ArrowUp".to_owned(),
            hard_drop: "Space".to_owned(),
            soft_drop: "ArrowDown".to_owned(),
            hold: "KeyC".to_owned(),
        }
    }
}

impl InputSource for KeyboardInput {
    fn controller(&self) -> Controller {
        let keys = self.keys.borrow();
        Controller {
            left: keys.contains(&self.config.left),
            right: keys.contains(&self.config.right),
            rotate_left: keys.contains(&self.config.rotate_left),
            rotate_right: keys.contains(&self.config.rotate_right),
            hard_drop: keys.contains(&self.config.hard_drop),
            soft_drop: keys.contains(&self.config.soft_drop),
            hold: keys.contains(&self.config.hold),
        }
    }
}

pub struct BotInput {
    interface: cold_clear::Interface,
    executing: Option<(FallingPiece, PieceMoveExecutor)>,
    controller: Controller,
    speed_limit: u32
}

impl BotInput {
    pub fn new(interface: cold_clear::Interface, speed_limit: u32) -> Self {
        BotInput {
            interface,
            executing: None,
            controller: Default::default(),
            speed_limit,
        }
    }
}

impl InputSource for BotInput {
    fn controller(&self) -> Controller {
        self.controller
    }

    fn update(
        &mut self, board: &Board<ColoredRow>, events: &[Event], incoming: u32
    ) -> Option<cold_clear::Info> {
        for event in events {
            match event {
                Event::PieceSpawned { new_in_queue } => {
                    self.interface.add_next_piece(*new_in_queue);
                    if self.executing.is_none() {
                        self.interface.request_next_move(incoming);
                    }
                }
                Event::GarbageAdded(_) => {
                    self.interface.reset(board.get_field(), board.b2b_bonus, board.combo);
                }
                _ => {}
            }
        }
        let mut info = None;
        if let Ok((mv, i)) = self.interface.poll_next_move() {
            info = Some(i);
            self.executing = Some((
                mv.expected_location,
                PieceMoveExecutor::new(mv.hold, mv.inputs.into_iter().collect(), self.speed_limit)
            ));
        }
        if let Some((expected, ref mut executor)) = self.executing {
            if let Some(loc) = executor.update(&mut self.controller, board, events) {
                if loc != expected {
                    self.interface.reset(board.get_field(), board.b2b_bonus, board.combo);
                }
                self.executing = None;
            }
        }
        info
    }
}
