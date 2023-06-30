use wasm_bindgen::prelude::*;

use crate::ion::Ion;

#[wasm_bindgen(js_name = Ion)]
pub struct IonJs {
    pub x: f64,
    pub y: f64,
}

impl IonJs {
    pub fn new(ion: &Ion) -> IonJs {
        IonJs {
            x: ion.pos.x,
            y: ion.pos.y,
        }
    }
}
