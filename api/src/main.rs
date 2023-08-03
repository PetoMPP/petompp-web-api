use lazy_static::lazy_static;
use petompp_web_api::{build_rocket, get_connection_pool, PgPool};

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    lazy_static! {
        static ref USER_REPO: PgPool = get_connection_pool();
    }
    build_rocket(&*USER_REPO)
}
