use wasm_bindgen::JsCast;

pub struct Ended(web_sys::Event);

impl webutil::event::Event for Ended {
    const NAME: &'static str = "ended";

    fn from_event(e: web_sys::Event) -> Self {
        Ended(e.unchecked_into())
    }
}

impl std::ops::Deref for Ended {
    type Target = web_sys::Event;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
