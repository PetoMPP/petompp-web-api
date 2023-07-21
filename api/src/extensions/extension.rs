pub struct Extension<T>(pub T);

impl<T> Extension<T> {
    pub fn into(self) -> T {
        self.0
    }
}
