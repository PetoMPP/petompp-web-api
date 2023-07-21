pub trait Controller {
    fn path(&self) -> &'static str;
    fn routes(&self) -> Vec<rocket::Route>;
}

pub trait ControllerRegisterer {
    fn add(self, controller: impl Controller) -> Self;
}