use std::path::Path;

pub fn main() -> Result<(), pluto_templating::RucteError> {
    let views_dir = "views/";
    let static_dir = "static/";
    
    let out_dir = std::env::var("OUT_DIR").expect("No source path set.");
    let out_path = Path::new(&out_dir);

    pluto_templating::initialize_templating_engine(out_path.to_path_buf(), views_dir, static_dir)
}
