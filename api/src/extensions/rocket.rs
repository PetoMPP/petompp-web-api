use super::extension::Extension;
use crate::controllers::controller::{Controller, ControllerRegisterer};
use rocket::{Build, Rocket};

impl ControllerRegisterer for Extension<Rocket<Build>> {
    fn add(self, controller: impl Controller) -> Self {
        const PATH: &'static str = "/api/v1";
        let path = controller.path();
        let path = match path.starts_with('/') {
            true => format!("{}{}", PATH, path),
            false => format!("{}/{}", PATH, path),
        };
        Extension(self.0.mount(path, controller.routes()))
    }
}
