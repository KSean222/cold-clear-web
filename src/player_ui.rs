use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::utils;
use crate::resources::Resources;

use libtetris::*;

pub struct PlayerUi {
    element: web_sys::Element,
    board_canvas: web_sys::HtmlCanvasElement,
    board_context: web_sys::CanvasRenderingContext2d,
    queue_canvas: web_sys::HtmlCanvasElement,
    queue_context: web_sys::CanvasRenderingContext2d,
    stats: web_sys::Element,
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

        let board_canvas: web_sys::HtmlCanvasElement = document
            .create_element("canvas")
            .unwrap()
            .dyn_into()
            .unwrap();
        board_canvas.set_class_name("board");
        container.append_child(&board_canvas).unwrap();

        let board_context: web_sys::CanvasRenderingContext2d = board_canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        let queue_canvas: web_sys::HtmlCanvasElement = document
            .create_element("canvas")
            .unwrap()
            .dyn_into()
            .unwrap();
        queue_canvas.set_class_name("queue");
        container.append_child(&queue_canvas).unwrap();

        let queue_context: web_sys::CanvasRenderingContext2d = queue_canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        Self {
            element,
            board_canvas,
            board_context,
            queue_canvas,
            queue_context,
            stats
        }
    }
    pub fn element(&self) -> &web_sys::Element {
        &self.element
    }
    pub fn update(&mut self) {

    }
    pub fn render(&mut self, resources: &Resources) {
        set_size_to_css_size(&self.board_canvas);
        set_size_to_css_size(&self.queue_canvas);
        for y in 0..21 {
            for x in 0..10 {
                self.draw_cell(resources, CellColor::Empty, x, y)
            }
        }
    }
    fn draw_cell(&self, resources: &Resources, cell: CellColor, x: i32, y: i32) {
        let src_cell_size = resources.skin.height() as f64;
        let dest_cell_size = self.board_canvas.height() as f64 / BOARD_HEIGHT;
        let cell_index = match cell {
            CellColor::Garbage => 1,
            CellColor::Z => 2,
            CellColor::L => 3,
            CellColor::O => 4,
            CellColor::S => 5,
            CellColor::I => 6,
            CellColor::J => 7,
            CellColor::T => 8,
            _ => 0
        };
        self.board_context
            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &resources.skin,
                0.0,
                cell_index as f64 * src_cell_size,
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


