#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Rule {
    pub born: u16,
    pub survives: u16,
    name: &'static str,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Rule {
    pub(crate) fn rule_array(&self) -> [u32; 2] {
        [u32::from(self.born), u32::from(self.survives)]
    }

    pub fn name(&self) -> String {
        let mut born = String::from("B");
        let mut survives = String::from("S");
        for i in 0..9 {
            if self.born & (1 << i) != 0 {
                born.push_str(&format!("{i}"));
            }
            if self.survives & (1 << i) != 0 {
                survives.push_str(&format!("{i}"));
            }
        }
        format!("{} {}/{}", self.name, born, survives)
    }
}

pub static RULES: [Rule; 9] = [
    Rule {
        born: 0b1000,
        survives: 0b1100,
        name: "Conway's Life",
    },
    Rule {
        born: 0b0_0000_1000,
        survives: 0b0_0011_1110,
        name: "Maze",
    },
    Rule {
        born: 0b0_0000_1000,
        survives: 0b0_0001_1110,
        name: "Mazectric",
    },
    Rule {
        born: 0b1,
        survives: 0b1,
        name: "Gnarl",
    },
    Rule {
        born: 0b1_1100_1000,
        survives: 0b1_1101_1000,
        name: "Day & Night",
    },
    Rule {
        born: 0b0_0010_1000,
        survives: 0b1_1011_1100,
        name: "Land Rush",
    },
    Rule {
        born: 0b0_0100_1000,
        survives: 0b1_1011_1100,
        name: "Land Rush 2",
    },
    Rule {
        born: 0b1_1100_1000,
        survives: 0b1_1110_1100,
        name: "Stains",
    },
    Rule {
        born: 0b1_1110_0000,
        survives: 0b1_1111_0000,
        name: "Vote",
    },
];

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = getRules)]
pub fn get_rules() -> wasm_bindgen::prelude::JsValue {
    wasm_bindgen::prelude::JsValue::from(
        RULES
            .iter()
            .copied()
            .map(wasm_bindgen::prelude::JsValue::from)
            .collect::<js_sys::Array>(),
    )
}
