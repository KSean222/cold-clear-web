// Required by `cold-clear`
importScripts("./main.js");
wasm_bindgen("./main_bg.wasm").then(() => wasm_bindgen._web_worker_entry_point(self));
