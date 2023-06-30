use js_sys::Array;
use wasm_bindgen::prelude::*;

use crate::{crystal_structure::CrystalStructure, electron_js::ElectronJs, ion_js::IonJs};

#[wasm_bindgen(js_name = CrystalStructure)]
pub struct CrystalStructureJs {
    cs: CrystalStructure,
}

#[wasm_bindgen(js_class = CrystalStructure)]
impl CrystalStructureJs {
    #[wasm_bindgen(constructor)]
    pub fn new(
        x_size: f64,
        y_size: f64,
        ion_distance: f64,
        init_velocity: f64,
        num_electrons: i32,
    ) -> CrystalStructureJs {
        CrystalStructureJs {
            cs: CrystalStructure::new(x_size, y_size, ion_distance, init_velocity, num_electrons),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn x_size(&self) -> f64 {
        self.cs.x_size
    }

    #[wasm_bindgen(getter)]
    pub fn y_size(&self) -> f64 {
        self.cs.x_size
    }

    #[wasm_bindgen(getter)]
    pub fn elec_left(&self) -> i32 {
        self.cs.elec_left
    }

    #[wasm_bindgen(getter)]
    pub fn elec_right(&self) -> i32 {
        self.cs.elec_right
    }

    pub fn update(&mut self, acc: f64, supp: f64) {
        self.cs.update(acc, supp);
    }

    pub fn get_ions(&self) -> Array {
        self.cs
            .ions
            .iter()
            .map(|ion| JsValue::from(IonJs::new(&ion.borrow())))
            .collect()
    }

    pub fn get_electrons(&self) -> Array {
        self.cs
            .electrons
            .iter()
            .map(|electron| JsValue::from(ElectronJs::new(&electron.borrow())))
            .collect()
    }

    pub fn avg_ticks_between_bounces(&self) -> f64 {
        let sum = self
            .cs
            .electrons
            .iter()
            .fold(0.0, |acc, el| acc + el.borrow().avg_ticks_between_bounces);

        sum / self.cs.electrons.len() as f64
    }
}
