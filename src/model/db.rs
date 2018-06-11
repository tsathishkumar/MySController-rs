use actix::*;
use diesel::prelude::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};

pub struct ConnDsl(pub Pool<ConnectionManager<SqliteConnection>>);

impl Actor for ConnDsl {
    type Context = SyncContext<Self>;
}
