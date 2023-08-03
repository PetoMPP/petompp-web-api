use rocket::{Build, Rocket};

pub trait Controller {
    fn path(&self) -> &'static str;
    fn routes(&self) -> Vec<rocket::Route>;
    fn add_managed(&self, rocket: rocket::Rocket<rocket::Build>) -> rocket::Rocket<rocket::Build>;
}

pub trait ControllerRegisterer {
    fn add(self, controller: impl Controller) -> Self;
}

impl ControllerRegisterer for Rocket<Build> {
    fn add(self, controller: impl Controller) -> Self {
        const PATH: &'static str = "/api/v1";
        let path = controller.path();
        let path = match path.starts_with('/') {
            true => format!("{}{}", PATH, path),
            false => format!("{}/{}", PATH, path),
        };
        controller
            .add_managed(self)
            .mount(path, controller.routes())
    }
}
