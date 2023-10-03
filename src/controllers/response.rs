use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<'a, T: Serialize> {
    pub status: &'a str,
    pub data: T,
}

impl<'a, T: Serialize> ApiResponse<'a, T> {
    pub fn ok(data: T) -> Self {
        Self {
            status: "success",
            data,
        }
    }

    pub fn err(data: T) -> Self {
        Self {
            status: "error",
            data,
        }
    }
}
