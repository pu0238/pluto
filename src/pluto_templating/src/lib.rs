use std::path::PathBuf;

/// This is a method that initializes the templating engine in a Pluto project.
///
/// Typically it is meant to be used in a build script (`build.rs` file).
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