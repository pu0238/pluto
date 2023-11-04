pub fn main() -> Result<(), pluto_templating::RucteError> {
    pluto_templating::initialize_templating_engine("views")
}
