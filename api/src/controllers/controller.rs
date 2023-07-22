pub trait Controller {
    fn path(&self) -> &'static str;
    fn routes(&self) -> Vec<rocket::Route>;
    fn add_managed(&self, rocket: rocket::Rocket<rocket::Build>) -> rocket::Rocket<rocket::Build>;
}

pub trait ControllerRegisterer {
    fn add(self, controller: impl Controller) -> Self;
}
