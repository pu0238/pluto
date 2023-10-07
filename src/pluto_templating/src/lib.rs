use std::path::PathBuf;

pub fn initialize_templating_engine(templates_dir: &str) -> Result<(), ructe::RucteError> {
    let out_dir = format!("{}", std::env::var("OUT_DIR").expect("No source path set."));
    let mut engine = ructe::Ructe::new(PathBuf::from(out_dir)).unwrap();
    match engine.compile_templates(templates_dir) {
        Err(_) => return Ok(()),
        _ => {}
    };
    match engine.statics() {
        Err(_) => return Ok(()),
        _ => {}
    };
    Ok(())
}
