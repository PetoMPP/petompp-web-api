use rocket::{Build, Rocket};

pub trait Controller {
    fn path(&self) -> &'static str;
    fn routes(&self) -> Vec<rocket::Route>;
}

pub trait ControllerRegisterer {
    fn add(self, controller: impl Controller) -> Self;
}

impl ControllerRegisterer for Rocket<Build> {
    fn add(self, controller: impl Controller) -> Self {
        const PATH: &str = "/api/v1";
        let path = controller.path();
        let path = match path.starts_with('/') {
            true => format!("{}{}", PATH, path),
            false => format!("{}/{}", PATH, path),
        };
        self.mount(path, controller.routes())
    }
}
