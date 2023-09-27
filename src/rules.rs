#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy)]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub struct Rule {
    pub born: u16,
    pub survives: u16,
    pub initial_density: u8,
    name: &'static str,
}

#[cfg_attr(target_family = "wasm", wasm_bindgen)]
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

pub static RULES: [Rule; 17] = [
    Rule {
        born: 0b1000,
        survives: 0b1100,
        name: "Conway's Life",
        initial_density: 12,
    },
    Rule {
        born: 0b0_1011_1000,
        survives: 0b1_0111_0000,
        name: "Gems",
        initial_density: 15,
    },
    Rule {
        born: 0b1_1100_1000,
        survives: 0b1_1101_1000,
        name: "Day & Night",
        initial_density: 50,
    },
    Rule {
        born: 0b1_1110_1000,
        survives: 0b1_1110_0000,
        name: "Diamoeba",
        initial_density: 48,
    },
    Rule {
        born: 0b0_1000_1000,
        survives: 0b0_0000_1100,
        name: "DryLife",
        initial_density: 12,
    },
    Rule {
        born: 0b1_1110_0100,
        survives: 0b1_1110_0000,
        name: "Iceballs	",
        initial_density: 1,
    },
    Rule {
        born: 0b0_0010_1000,
        survives: 0b1_1011_1100,
        name: "Land Rush",
        initial_density: 4,
    },
    Rule {
        born: 0b0_0100_1000,
        survives: 0b1_1011_1100,
        name: "Land Rush 2",
        initial_density: 4,
    },
    Rule {
        born: 0b0_0000_0100,
        survives: 0b0_0000_0000,
        name: "Live Free or Die	",
        initial_density: 1,
    },
    Rule {
        born: 0b0_0000_1000,
        survives: 0b0_0011_1110,
        name: "Maze",
        initial_density: 3,
    },
    Rule {
        born: 0b0_0000_1000,
        survives: 0b0_0001_1110,
        name: "Mazectric",
        initial_density: 3,
    },
    Rule {
        born: 0b1_0000_1000,
        survives: 0b0_0000_1100,
        name: "Pedestrian Life",
        initial_density: 10,
    },
    Rule {
        born: 0b0_1010_1010,
        survives: 0b0_1010_1010,
        name: "Replicator",
        initial_density: 1,
    },
    Rule {
        born: 0b0_0001_1100,
        survives: 0b0_0000_0000,
        name: "Serviettes",
        initial_density: 1,
    },
    Rule {
        born: 0b0_0000_1000,
        survives: 0b0_1000_1110,
        name: "SnowLife",
        initial_density: 3,
    },
    Rule {
        born: 0b1_1100_1000,
        survives: 0b1_1110_1100,
        name: "Stains",
        initial_density: 8,
    },
    Rule {
        born: 0b1_1110_0000,
        survives: 0b1_1111_0000,
        name: "Vote",
        initial_density: 50,
    },
];

#[cfg(target_family = "wasm")]
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
