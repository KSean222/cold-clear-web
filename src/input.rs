use std::collections::HashSet;

use crate::utils;

use libtetris::*;

pub trait InputSource {
    fn fetch_inputs(&mut self) -> Controller;
}

pub struct InputConfig<T> {
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
    keys: HashSet<String>
}
// TODO
// impl KeyboardInput {
//     pub fn new(config: InputConfig<String>) -> Self {
//         let window = utils::window();
        
//         window.add_event_listener_with_callback();
//         Self {
//             config,
//             keys: 
//         }
//     }
// }

// impl Default for KeyboardInput {
//     fn default() -> Self {
//         Self {
//             config: InputConfig {
//                 left: "ArrowLeft",
//                 right: "ArrowRight",
//                 rotate_left: "KeyX",
//                 rotate_right: "ArrowUp",
//                 hard_drop: "Space",
//                 soft_drop: "ArrowDown",
//                 hold: "KeyC",
//             },
//             keys: HashSet::new()
//         }
//     }
// }

// impl InputSource for KeyboardInput {
//     fn fetch_inputs() -> Controller {
//         Controller {
//             left: "ArrowLeft",
//             right: "ArrowRight",
//             rotate_left: "KeyX",
//             rotate_right: "ArrowUp",
//             hard_drop: "Space",
//             soft_drop: "ArrowDown",
//             hold: "KeyC",
//         }
//     }
// }
