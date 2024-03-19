pub use ructe::RucteError;
use std::path::PathBuf;

/// This is a method that initializes the templating engine in a Pluto project.
///
/// Typically it is meant to be used in a build script (`build.rs` file).
pub fn initialize_templating_engine(
    out_dir: PathBuf,
    templates_dir: &str,
    static_dir: &str,
) -> Result<(), ructe::RucteError> {
    let mut engine = ructe::Ructe::new(out_dir)?;
    engine.compile_templates(templates_dir)?;

    let mut statics = engine.statics()?;
    statics.add_files_as(static_dir, "")?;
    Ok(())
}
