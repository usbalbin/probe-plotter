use probe_plotter_common::symbol::Symbol;

fn main() {
    let sym = Symbol::Setting {
        name: "SETTING".to_string(),
        ty: probe_plotter_common::PrimitiveType::i8,
        range: -1.0..=7.0,
        step_size: 2.0,
    };
    let s = r#"{
        "type":"Setting",
        "name":"SETTING",
        "ty":"i8",
        "range":{"start":1.0,"end":7.0},
        "step_size":2.0
    }"#;
    //assert_eq!(serde_json::to_string(&sym).unwrap(), s);
    Symbol::demangle(s).unwrap();
}
