use wasm_bindgen::prelude::*;

use crate::electron::Electron;

#[wasm_bindgen(js_name = Electron)]
pub struct ElectronJs {
    pub x: f64,
    pub y: f64,
    pub avg_ticks_between_bounces: f64,
}

impl ElectronJs {
    pub fn new(electron: &Electron) -> Self {
        ElectronJs {
            x: electron.pos.x,
            y: electron.pos.y,
            avg_ticks_between_bounces: electron.avg_ticks_between_bounces,
        }
    }
}
