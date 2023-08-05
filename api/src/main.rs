use diesel::{Connection, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use lazy_static::lazy_static;
use petompp_web_api::{build_rocket, get_connection_pool, PgPool, Secrets};

#[macro_use]
extern crate rocket;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[launch]
fn rocket() -> _ {
    lazy_static! {
        static ref SECRETS: Secrets = Secrets::default();
        static ref USER_REPO: PgPool = get_connection_pool(&SECRETS);
    }

    {
        let mut conn = PgConnection::establish(&SECRETS.database_url).unwrap();
        conn.run_pending_migrations(MIGRATIONS).unwrap();
    }
    build_rocket(&SECRETS, &*USER_REPO)
}
