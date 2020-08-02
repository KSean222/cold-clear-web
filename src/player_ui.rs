use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::utils;
use crate::resources::Resources;

use libtetris::*;
use battle::{Game, Event};
use arrayvec::ArrayVec;

const ATTACK_TYPE_TEXT_TIME: f64 = 1.0;
const ATTACK_TYPE_TEXT_FADE: f64 = 0.25;

fn opacity(then: u32, now: u32) -> f64 {
    let fade = (now - then).saturating_sub((ATTACK_TYPE_TEXT_TIME * crate::UPS) as u32);
    1.0 - (fade as f64 / ATTACK_TYPE_TEXT_FADE / crate::UPS).min(1.0)
}

pub struct PlayerUi {
    element: web_sys::Element,
    board_canvas: web_sys::HtmlCanvasElement,
    board_context: web_sys::CanvasRenderingContext2d,
    queue_canvas: web_sys::HtmlCanvasElement,
    queue_context: web_sys::CanvasRenderingContext2d,
    hold_canvas: web_sys::HtmlCanvasElement,
    hold_context: web_sys::CanvasRenderingContext2d,
    stats: web_sys::Element,
    garbage_bar: web_sys::HtmlElement,
    attack_type_text: web_sys::HtmlElement,
    combo_text: web_sys::HtmlElement,
    board: Board<ColoredRow>,
    state: PlayerState,
    last_attack_type: Option<(LockResult, u32)>,
    last_combo: Option<(u32, u32)>,
    time: u32
}

enum PlayerState {
    Falling(FallingPiece, FallingPiece),
    SpawnDelay,
    LineClearDelay {
        elapsed: u32,
        lines: ArrayVec<[i32; 4]>,
        piece: FallingPiece
    },
    GameOver
}

// Defined in terms of cells
const STATS_WIDTH: f64 = 4.0;
const BOARD_WIDTH: f64 = 10.0;
const QUEUE_WIDTH: f64 = 3.0;
const BOARD_HEIGHT: f64 = 20.5;

fn set_size_to_css_size(canvas: &web_sys::HtmlCanvasElement) {
    let element: &web_sys::HtmlElement = canvas.dyn_ref().unwrap();
    canvas.set_width(element.client_width().max(0) as u32);
    canvas.set_height(element.client_height().max(0) as u32);
}

impl PlayerUi {
    pub fn new() -> Self {
        let document = utils::document();

        let element = document
            .create_element("div")
            .unwrap();
        element.set_class_name("player");

        let container = document
            .create_element("div")
            .unwrap();
        container.set_class_name("board-container");
        element.append_child(&container).unwrap();

        let stats = document
            .create_element("div")
            .unwrap();
        stats.set_class_name("stats");
        container.append_child(&stats).unwrap();

        let (board_canvas, board_context) = utils::new_canvas();
        board_canvas.set_class_name("board");
        container.append_child(&board_canvas).unwrap();

        let (queue_canvas, queue_context) = utils::new_canvas();
        queue_canvas.set_class_name("queue");
        container.append_child(&queue_canvas).unwrap();

        let (hold_canvas, hold_context) = utils::new_canvas();
        hold_canvas.set_class_name("hold");
        container.append_child(&hold_canvas).unwrap();

        let garbage_bar_container = document
            .create_element("div")
            .unwrap();
        garbage_bar_container.set_class_name("garbage-bar");
        container.append_child(&garbage_bar_container).unwrap();

        let garbage_bar: web_sys::HtmlElement = document
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        garbage_bar_container.append_child(&garbage_bar).unwrap();

        let attack_type_text: web_sys::HtmlElement = document
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        attack_type_text.set_class_name("attack-text attack-type-text");
        container.append_child(&attack_type_text).unwrap();

        let combo_text: web_sys::HtmlElement = document
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        combo_text.set_class_name("attack-text combo-text");
        container.append_child(&combo_text).unwrap();
        
        Self {
            element,
            board_canvas,
            board_context,
            queue_canvas,
            queue_context,
            hold_canvas,
            hold_context,
            garbage_bar,
            attack_type_text,
            combo_text,
            stats,
            board: Board::new(),
            state: PlayerState::SpawnDelay,
            last_attack_type: None,
            last_combo: None,
            time: 0
        }
    }
    pub fn element(&self) -> &web_sys::Element {
        &self.element
    }
    pub fn update(&mut self, game_events: &[Event]) {
        self.time += 1;
        match &mut self.state {
            PlayerState::LineClearDelay { elapsed, .. } => {
                *elapsed += 1;
            }
            _ => {}
        }
        for event in game_events {
            match event {
                &Event::GameOver => {
                    self.state = PlayerState::GameOver;
                }
                &Event::PieceFalling(piece, ghost) => {
                    self.state = PlayerState::Falling(piece, ghost);
                }
                Event::GarbageAdded(columns) => {
                    for &column in columns {
                        self.board.add_garbage(column);
                    }
                }
                Event::PiecePlaced { piece, locked, .. } => {
                    if locked.cleared_lines.is_empty() {
                        self.board.lock_piece(*piece);
                        self.state = PlayerState::SpawnDelay;
                    } else {
                        self.state = PlayerState::LineClearDelay {
                            elapsed: 0,
                            lines: locked.cleared_lines.clone(),
                            piece: *piece
                        };
                    }
                    if locked.placement_kind.is_hard() || locked.perfect_clear {
                        self.last_attack_type = Some((locked.clone(), self.time));
                    }
                    if let Some(combo) = locked.combo {
                        if combo > 0 {
                            self.last_combo = Some((combo, self.time));
                        }
                    }
                },
                Event::EndOfLineClearDelay => {
                    if let PlayerState::LineClearDelay { piece, .. } = self.state {
                        self.board.lock_piece(piece);
                        self.state = PlayerState::SpawnDelay;
                    }
                }
                _ => {}
            }
        }
    }
    pub fn render(&self, resources: &Resources, game: &Game) {
        set_size_to_css_size(&self.board_canvas);
        set_size_to_css_size(&self.queue_canvas);
        set_size_to_css_size(&self.hold_canvas);
        for y in 0..21 {
            let row = self.board.get_row(y);
            for x in 0..10 {
                let mut col = row.cell_color(x as usize);
                if col != CellColor::Empty {
                    if let PlayerState::GameOver = self.state {
                        col = CellColor::Unclearable;
                    }
                }
                self.draw_cell(resources, col, false, x, y);
            }
        }
        match &self.state {
            &PlayerState::Falling(piece, ghost) => {
                self.draw_piece(resources, ghost, true);
                self.draw_piece(resources, piece, false);
            }
            &PlayerState::LineClearDelay { piece, .. } => {
                self.draw_piece(resources, piece, false);
            }
            _ => {}
        }
        self.garbage_bar
            .style()
            .set_property("height", &format!("calc({} / {} * 100%)", game.garbage_queue, BOARD_HEIGHT))
            .unwrap();
        
        
        if let Some((lock, time)) = &self.last_attack_type {
            if lock.perfect_clear {
                self.attack_type_text.set_inner_text("Perfect Clear");
            } else if lock.b2b {
                self.attack_type_text.set_inner_text(&format!("Back-To-Back {}", lock.placement_kind.name()));
            } else {
                self.attack_type_text.set_inner_text(lock.placement_kind.name());
            }
            self.attack_type_text
                .style()
                .set_property("opacity", &format!("{}", opacity(*time, self.time)))
                .unwrap();
        }
        if let Some((combo, time)) = self.last_combo {
            self.combo_text.set_inner_text(&format!("{} combo", combo));
            self.combo_text
                .style()
                .set_property("opacity", &format!("{}", opacity(time, self.time)))
                .unwrap();
        }
    }
    fn draw_piece(&self, resources: &Resources, piece: FallingPiece, is_ghost: bool) {
        for &(x, y) in &piece.cells() {
            self.draw_cell(resources, piece.kind.0.color(), is_ghost, x, y);
        }
    }
    fn draw_cell(&self, resources: &Resources, cell: CellColor, is_ghost: bool, x: i32, y: i32) {
        let src_cell_size = (resources.skin.height() / 2) as f64;
        let dest_cell_size = self.board_canvas.height() as f64 / BOARD_HEIGHT;
        let cell_x = match cell {
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
        let cell_y = if is_ghost { 1 } else { 0 };
        self.board_context
            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &resources.skin,
                cell_x as f64 * src_cell_size,
                cell_y as f64 * src_cell_size,
                src_cell_size,
                src_cell_size,
                x as f64 * dest_cell_size,
                (BOARD_HEIGHT - (y + 1) as f64) * dest_cell_size,
                dest_cell_size,
                dest_cell_size
            )
            .unwrap();
    }
}


