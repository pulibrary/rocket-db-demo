use diesel::deserialize::Queryable;
use rocket::fairing::AdHoc;
use rocket::response::{Debug, status::Created, status::NoContent};
use rocket::serde::{Serialize, Deserialize, json::Json};
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use rocket_db_pools::{Connection, Database};
use rocket_db_pools::diesel::{PgPool, prelude::*};

// Imports for running migrations on startup
// use rocket::fairing;
// use diesel::Connection as SyncConnection;
// use diesel::pg::{Pg, PgConnection};
// use rocket::{Rocket, Build};
// use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
// use rocket::fairing::{Fairing, Info, Kind};

type Result<T, E = Debug<diesel::result::Error>> = std::result::Result<T, E>;

#[derive(Database)]
#[database("diesel_postgres")]
struct Db(PgPool);

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Insertable)]
#[serde(crate = "rocket::serde")]
#[diesel(table_name = notes)]
struct Note {
    id: Option<i64>,
    title: String,
    message: String,
}

diesel::table! {
    notes (id) {
        id -> Nullable<BigInt>,
        title -> Varchar,
        message -> Varchar,
    }
}

#[post("/", data = "<note>")]
async fn create(mut db: Connection<Db>, note: Json<Note>) -> Result<Created<Json<Note>>> {

    let result = db.transaction(|conn| Box::pin(async move {
        diesel::insert_into(notes::table)
            .values(&*note)
            .get_result::<Note>(conn)
            .await
            .map_err(Debug)
    })).await;

    match result {
        Ok(note) => Ok(Created::new("/").body(Json(note))),
        Err(e) => Err(e),
    }
}

#[get("/")]
async fn list(mut db: Connection<Db>) -> Result<Json<Vec<Note>>> {
    let notes = notes::table
        .load::<Note>(&mut db)
        .await?;

    Ok(Json(notes))
}

#[get("/<id>")]
async fn read(mut db: Connection<Db>, id: i64) -> Option<Json<Note>> {
    notes::table
        .filter(notes::id.eq(id))
        .first(&mut db)
        .await
        .map(Json)
        .ok()
}

#[delete("/<id>")]
async fn delete(mut db: Connection<Db>, id: i64) -> Result<NoContent, rocket::response::Debug<diesel::result::Error>> {
    diesel::delete(notes::table)
        .filter(notes::id.eq(id))
        .execute(&mut **db)
        .await?;

    Ok(NoContent)
}

// TODO: Fix migration on startup
// pub struct RunMigrationsFairing;

// #[rocket::async_trait]
// impl Fairing for RunMigrationsFairing {
//     fn info(&self) -> Info {
//         Info {
//             name: "Run Postgres Database Migrations",
//             kind: Kind::Ignite,
//         }
//     }

//     async fn on_ignite(&self, rocket: Rocket<Build>) -> rocket::fairing::Result {
//         let db_url = rocket.figment().extract_inner::<String>("databases.diesel_postgres.url").unwrap();
//         const MIGRATIONS: EmbeddedMigrations = embed_migrations!("db/diesel/migrations");

//         match SyncConnection::establish(&db_url) {
//             Ok(mut conn:) => {
//                 match conn.run_pending_migrations(MIGRATIONS) {
//                     Ok(_) => Ok(rocket),
//                     Err(e) => Err(rocket)
//                 }
//             }
//             Err(e) => Err(rocket)
//         }
//     }
// }

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Diesel Postgres Stage", |rocket| async {
        rocket.attach(Db::init())
            // TODO: Fix migration on startup
            // .attach(RunMigrationsFairing)
            .mount("/diesel", routes![list, create, read, delete])
    })
}
