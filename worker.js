// Required by `cold-clear`

class AudioContext {} // Work around `wasm-bindgen` acquiring the AudioContext class in web workers.
importScripts("./main.js");
wasm_bindgen("./main_bg.wasm").then(() => wasm_bindgen._web_worker_entry_point(self));
