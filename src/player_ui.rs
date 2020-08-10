use wasm_bindgen::{JsCast, JsValue};
use webutil::event::EventTargetExt;

use crate::utils;
use crate::resources::Resources;
use crate::audio_ended_event;
use crate::options::PlayerConfig;

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
    statistics_text_name: web_sys::HtmlElement,
    statistics_text_value: web_sys::HtmlElement,
    garbage_bar: web_sys::HtmlElement,
    attack_type_text: web_sys::HtmlElement,
    combo_text: web_sys::HtmlElement,
    name_text: web_sys::HtmlElement,
    move_sfx_finished: Option<webutil::event::EventOnce<audio_ended_event::Ended>>,
    board: Board<ColoredRow>,
    statistics: Statistics,
    state: PlayerState,
    last_attack_type: Option<(LockResult, u32)>,
    last_combo: Option<(u32, u32)>,
    info: Option<cold_clear::Info>,
    time: u32
}

enum PlayerState {
    Falling(FallingPiece, FallingPiece),
    SpawnDelay,
    LineClearDelay {
        time: u32,
        lines: ArrayVec<[i32; 4]>,
        piece: FallingPiece
    },
    GameOver
}

// Defined in terms of cells
const HOLD_WIDTH: f64 = 4.0;
const QUEUE_WIDTH: f64 = 3.0;
const BOARD_HEIGHT: f64 = 20.5;

fn set_size_to_css_size(canvas: &web_sys::HtmlCanvasElement) {
    let element: &web_sys::HtmlElement = canvas.dyn_ref().unwrap();
    canvas.set_width(element.client_width().max(0) as u32);
    canvas.set_height(element.client_height().max(0) as u32);
}

impl PlayerUi {
    pub fn new(name: String) -> Self {
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

        let statistics_text_label: web_sys::HtmlElement = document
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        statistics_text_label.set_class_name("stats-label");
        statistics_text_label.set_inner_text("Statistics");
        container.append_child(&statistics_text_label).unwrap();

        let statistics_text_name: web_sys::HtmlElement = document
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        statistics_text_name.set_class_name("stats-box stats-name");
        container.append_child(&statistics_text_name).unwrap();

        let statistics_text_value: web_sys::HtmlElement = document
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        statistics_text_value.set_class_name("stats-box stats-value");
        container.append_child(&statistics_text_value).unwrap();

        let (board_canvas, board_context) = utils::new_canvas();
        board_canvas.set_class_name("board");
        container.append_child(&board_canvas).unwrap();

        let (queue_canvas, queue_context) = utils::new_canvas();
        queue_canvas.set_class_name("queue-box");
        container.append_child(&queue_canvas).unwrap();
        
        let queue_text: web_sys::HtmlElement = document
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        queue_text.set_class_name("queue-box queue-text");
        queue_text.set_inner_text("Queue");
        container.append_child(&queue_text).unwrap();

        let (hold_canvas, hold_context) = utils::new_canvas();
        hold_canvas.set_class_name("hold-box");
        container.append_child(&hold_canvas).unwrap();

        let hold_text: web_sys::HtmlElement = document
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        hold_text.set_class_name("hold-box hold-text");
        hold_text.set_inner_text("Hold");
        container.append_child(&hold_text).unwrap();

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
        
        let name_text: web_sys::HtmlElement = document
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        name_text.set_id("name-text");
        name_text.set_inner_text(&name);
        container.append_child(&name_text).unwrap();
        
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
            name_text,
            statistics_text_name,
            statistics_text_value,
            move_sfx_finished: None,
            board: Board::new(),
            statistics: Statistics::default(),
            state: PlayerState::SpawnDelay,
            last_attack_type: None,
            last_combo: None,
            info: None,
            time: 0
        }
    }
    pub fn element(&self) -> &web_sys::Element {
        &self.element
    }
    pub fn update(&mut self, resources: &Resources, game_events: &[Event], info: Option<cold_clear::Info>) {
        if info.is_some() {
            self.info = info;
        }
        self.time += 1;
        for event in game_events {
            match event {
                Event::PieceMoved | Event::SoftDropped | Event::PieceRotated => {
                    let can_play = if let Some(event) = &self.move_sfx_finished {
                        event.try_next().is_some()
                    } else {
                        true
                    };
                    if can_play {
                        let event = utils::play_sound(&resources.audio_context, &resources.move_sfx)
                                .unwrap()
                                .dyn_into::<web_sys::EventTarget>()
                                .unwrap()
                                .once::<audio_ended_event::Ended>();
                        self.move_sfx_finished = Some(event);
                    }
                }
                Event::GameOver => {
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
                Event::PiecePlaced { piece, locked, hard_drop_distance } => {
                    self.statistics.update(locked);
                    if hard_drop_distance.is_some() {
                        utils::play_sound(&resources.audio_context, &resources.hard_drop_sfx).unwrap();
                    }
                    if locked.placement_kind.is_clear() {
                        utils::play_sound(&resources.audio_context, &resources.line_clear_sfx).unwrap();
                    }
                    if locked.cleared_lines.is_empty() {
                        self.board.lock_piece(*piece);
                        self.state = PlayerState::SpawnDelay;
                    } else {
                        self.state = PlayerState::LineClearDelay {
                            time: self.time,
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
                    self.board.advance_queue();
                }
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
    pub fn render(&self, resources: &Resources, game: &Game, config: &PlayerConfig) {
        self.draw_board(resources, config);
        self.draw_hold(resources, game);
        self.draw_queue(resources, game);
        self.draw_garbage_bar(game);
        self.draw_attack_type_text();
        self.draw_statistics();
    }
    fn draw_board(&self, resources: &Resources, config: &PlayerConfig) {
        set_size_to_css_size(&self.board_canvas);
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
            PlayerState::LineClearDelay { piece, lines, time } => {
                const OFFSET_FRAMES: u32 = 15;
                const INIT_WIDTH: f64 = 9.0;
                const TOTAL_WIDTH: f64 = 10.0;
                self.draw_piece(resources, *piece, false);
                let dest_cell_size = self.board_canvas.height() as f64 / BOARD_HEIGHT;
                let elapsed = self.time - *time;
                let line_clear_delay = config.game.line_clear_delay.saturating_sub(OFFSET_FRAMES).max(1) as f64;
                let rect_scale = (elapsed as f64 / line_clear_delay).min(1.0);
                let rect_width = dest_cell_size * (INIT_WIDTH + rect_scale);
                let rect_height = dest_cell_size * rect_scale;
                let cells_scale = (elapsed.saturating_sub(OFFSET_FRAMES) as f64 / line_clear_delay).min(1.0);
                let cells_width = dest_cell_size * (INIT_WIDTH + cells_scale);
                let cells_height = dest_cell_size * cells_scale;
                self.board_context.set_fill_style(&JsValue::from_str("white"));
                for &line in lines {
                    let (x, y) = self.cell_pos(0, line);
                    self.board_context.fill_rect(
                        x + (dest_cell_size * TOTAL_WIDTH - rect_width) / 2.0,
                        y + (dest_cell_size - rect_height) / 2.0,
                        rect_width,
                        rect_height
                    );
                    let cells_x = x + (dest_cell_size * TOTAL_WIDTH - cells_width) / 2.0;
                    let cells_y = y + (dest_cell_size - cells_height) / 2.0;
                    self.board_context.save();
                    self.board_context.begin_path();
                    self.board_context.move_to(cells_x, cells_y);
                    self.board_context.line_to(cells_x + cells_width, cells_y);
                    self.board_context.line_to(cells_x + cells_width, cells_y + cells_height);
                    self.board_context.line_to(cells_x, cells_y + cells_height);
                    self.board_context.clip();
                    for x in 0..10 {
                        self.draw_cell(resources, CellColor::Empty, false, x, line);
                    }
                    self.board_context.restore();
                }
                self.board_context.set_fill_style(&JsValue::from_str(""));
            }
            _ => {}
        }

        if let Some(info) = &self.info {
            let dest_cell_size = self.board_canvas.height() as f64 / BOARD_HEIGHT;

            let has_pc = info.plan.iter().any(|(_, lock)| lock.perfect_clear);

            let mut y_map = [0; 40];
            for i in 0..y_map.len() {
                y_map[i] = i as i32;
            }
            for (placement, lock) in &info.plan {
                for &(x, y, d) in &placement.cells_with_connections() {
                    let color = match placement.kind.0.color() {
                        CellColor::Z => "rgb(255, 32, 32)",
                        CellColor::S => "rgb(32, 255, 32)",
                        CellColor::O => "rgb(255, 255, 32)",
                        CellColor::L => "rgb(255, 143, 32)",
                        CellColor::J => "rgb(96, 96, 255)",
                        CellColor::I => "rgb(32, 255, 255)",
                        CellColor::T => "rgb(143, 32, 255)",
                        _ => ""
                    };
                    self.board_context.set_fill_style(&JsValue::from_str(color));
                    const LINE_WIDTH: f64 = 10.0 / 83.0;
                    const CELL_CORNERS: &[(f64, f64)] = &[
                        (0.0, 0.0),
                        (1.0 - LINE_WIDTH, 0.0),
                        (1.0 - LINE_WIDTH, 1.0 - LINE_WIDTH),
                        (0.0, 1.0 - LINE_WIDTH),
                    ];
                    let (x, y) = self.cell_pos(x, y_map[y as usize]);
                    for &(cell_x, cell_y) in CELL_CORNERS {
                        self.board_context.fill_rect(
                            cell_x * dest_cell_size + x,
                            cell_y * dest_cell_size + y,
                            LINE_WIDTH * dest_cell_size,
                            LINE_WIDTH * dest_cell_size
                        );
                    }
                    for d in d.complement().iter() {
                        let (cell_x, cell_y, cell_w, cell_h) = match d {
                            Direction::Up => (0.0, 0.0, 1.0, LINE_WIDTH),
                            Direction::Right => (1.0 - LINE_WIDTH, 0.0, LINE_WIDTH, 1.0),
                            Direction::Down => (0.0, 1.0 - LINE_WIDTH, 1.0, LINE_WIDTH),
                            Direction::Left => (0.0, 0.0, LINE_WIDTH, 1.0),
                        };
                        self.board_context.fill_rect(
                            cell_x * dest_cell_size + x,
                            cell_y * dest_cell_size + y,
                            cell_w * dest_cell_size,
                            cell_h * dest_cell_size
                        );
                    }
                    self.board_context.set_fill_style(&JsValue::from_str(""));
                }
                let mut new_map = [0; 40];
                let mut j = 0;
                for i in 0..40 {
                    if !lock.cleared_lines.contains(&i) {
                        new_map[j] = y_map[i as usize];
                        j += 1;
                    }
                }
                y_map = new_map;

                if !has_pc && lock.placement_kind.is_hard() && lock.placement_kind.is_clear()
                        || lock.perfect_clear {
                    break
                }
            }
        }
    }
    fn draw_hold(&self, resources: &Resources, game: &Game) {
        set_size_to_css_size(&self.hold_canvas);
        self.hold_context.clear_rect(0.0, 0.0, self.hold_canvas.width() as f64, self.hold_canvas.height() as f64);
        if let Some(piece) = game.board.hold_piece {
            let piece_canvas = &resources.pieces[piece];
            const PIECE_SCALE: f64 = 0.8;
            let scale = self.hold_canvas.width() as f64 / HOLD_WIDTH / resources.cell_size as f64;
            let width = piece_canvas.width() as f64 * PIECE_SCALE * scale;
            let height = piece_canvas.height() as f64 * PIECE_SCALE * scale;
            self.hold_context
                .draw_image_with_html_canvas_element_and_dw_and_dh(
                    piece_canvas,
                    (self.hold_canvas.width() as f64 - width) / 2.0,
                    (self.hold_canvas.height() as f64 - height) / 2.0,
                    width,
                    height
                )
                .unwrap();
        }
    }
    fn draw_queue(&self, resources: &Resources, game: &Game) {
        set_size_to_css_size(&self.queue_canvas);
        self.queue_context.clear_rect(0.0, 0.0, self.queue_canvas.width() as f64, self.hold_canvas.height() as f64);
        let queue_scale = self.queue_canvas.width() as f64 / QUEUE_WIDTH / resources.cell_size as f64;
        for (i, piece) in game.board.next_queue().enumerate() {
            let piece_canvas = &resources.pieces[piece];
            const PIECE_SCALE: f64 = 0.6;
            const SEPARATION: f64 = 0.8;
            let width = piece_canvas.width() as f64 * PIECE_SCALE * queue_scale;
            let height = piece_canvas.height() as f64 * PIECE_SCALE * queue_scale;
            let x = (self.queue_canvas.width() as f64 - width) / 2.0;
            let y = (self.queue_canvas.width() as f64 - height) / 2.0
                + i as f64 * self.queue_canvas.width() as f64 * SEPARATION;
            self.queue_context
                .draw_image_with_html_canvas_element_and_dw_and_dh(
                    piece_canvas,
                    x,
                    y,
                    width,
                    height
                )
                .unwrap();
        }
    }
    fn draw_garbage_bar(&self, game: &Game) {
        self.garbage_bar
            .style()
            .set_property("height", &format!("calc({} / {} * 100%)", game.garbage_queue, BOARD_HEIGHT))
            .unwrap();
    }
    fn draw_attack_type_text(&self) {
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
    fn draw_statistics(&self) {
        let seconds = self.time as f64 / crate::UPS;
        let mut lines = vec![
            ("Pieces", format!("{}", self.statistics.pieces)),
            ("PPS", format!("{:.1}", self.statistics.pieces as f64 / seconds)),
            ("Lines", format!("{}", self.statistics.lines)),
            ("Attack", format!("{}", self.statistics.attack)),
            ("APM", format!("{:.1}", self.statistics.attack as f64 / seconds * 60.0)),
            ("APP", format!("{:.3}", self.statistics.attack as f64 / self.statistics.pieces as f64)),
            ("Max Ren", format!("{}", self.statistics.max_combo)),
            ("Single", format!("{}", self.statistics.singles)),
            ("Double", format!("{}", self.statistics.doubles)),
            ("Triple", format!("{}", self.statistics.triples)),
            ("Tetris", format!("{}", self.statistics.tetrises)),
            // ("Mini T0", format!("{}", self.statistics.mini_tspin_zeros)),
            // ("Mini T1", format!("{}", self.statistics.mini_tspin_singles)),
            // ("Mini T2", format!("{}", self.statistics.mini_tspin_doubles)),
            ("T-Spin 0", format!("{}", self.statistics.tspin_zeros)),
            ("T-Spin 1", format!("{}", self.statistics.tspin_singles)),
            ("T-Spin 2", format!("{}", self.statistics.tspin_doubles)),
            ("T-Spin 3", format!("{}", self.statistics.tspin_triples)),
            ("Perfect", format!("{}", self.statistics.perfect_clears))
        ];
        if let Some(info) = &self.info {
            // Bot info
            lines.extend_from_slice(&[
                ("", "".to_owned()),
                ("Depth", format!("{}", info.depth)),
                ("Nodes", format!("{}", info.nodes)),
                ("O. Rank", format!("{}", info.original_rank))
            ]);
        }
        let (names, values): (Vec<_>, Vec<_>) = lines
            .into_iter()
            .map(|(n, v)| (n.to_owned(), v))
            .unzip();
        self.statistics_text_name.set_inner_text(&names.join("\n"));
        self.statistics_text_value.set_inner_text(&values.join("\n"));
    }
    fn draw_piece(&self, resources: &Resources, piece: FallingPiece, is_ghost: bool) {
        for &(x, y) in &piece.cells() {
            self.draw_cell(resources, piece.kind.0.color(), is_ghost, x, y);
        }
    }
    fn cell_pos(&self, x: i32, y: i32) -> (f64, f64) {
        let dest_cell_size = self.board_canvas.height() as f64 / BOARD_HEIGHT;
        (x as f64 * dest_cell_size, (BOARD_HEIGHT - (y + 1) as f64) * dest_cell_size)
    }
    fn draw_cell(&self, resources: &Resources, cell: CellColor, is_ghost: bool, x: i32, y: i32) {
        let dest_cell_size = self.board_canvas.height() as f64 / BOARD_HEIGHT;
        let (dest_cell_x, dest_cell_y) = self.cell_pos(x, y);
        let (src_cell_x, src_cell_y) = Resources::cell_pos(cell, is_ghost);
        self.board_context
            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &resources.skin,
                (src_cell_x * resources.cell_size) as f64,
                (src_cell_y * resources.cell_size) as f64,
                resources.cell_size as f64,
                resources.cell_size as f64,
                dest_cell_x,
                dest_cell_y,
                dest_cell_size,
                dest_cell_size
            )
            .unwrap();
    }
    pub fn reset(&mut self, name: String) {
        self.board = Board::new();
        self.statistics = Statistics::default();
        self.state = PlayerState::SpawnDelay;
        self.last_attack_type = None;
        self.attack_type_text.set_inner_text("");
        self.last_combo = None;
        self.combo_text.set_inner_text("");
        self.name_text.set_inner_text(&name);
        self.info = None;
        self.time = 0;
    }
}
