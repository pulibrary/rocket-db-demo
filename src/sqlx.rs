use rocket::fairing::AdHoc;
use rocket_db_pools::{sqlx, Database, Connection};
use rocket::response::status::Created;
use rocket::serde::{Serialize, Deserialize, json::Json};

use crate::rocket::futures::{TryFutureExt, TryStreamExt};

#[derive(Database)]
#[database("sqlx_postgres")]
struct Db(sqlx::PgPool);

type Result<T, E = rocket::response::Debug<sqlx::Error>> = std::result::Result<T, E>;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Note {
    id: Option<i64>,
    title: String,
    message: String,
}

#[post("/", data = "<note>")]
async fn create(mut db: Connection<Db>, mut note: Json<Note>) -> Result<Created<Json<Note>>> {
    let results = sqlx::query!(
        "INSERT INTO notes (title, message) VALUES ($1, $2) RETURNING id",
        note.title, note.message
    )
    .fetch(&mut **db)
    .try_collect::<Vec<_>>()
    .await?;

    note.id = Some(results.first().expect("returning results").id);
    Ok(Created::new("/").body(note))
}

#[get("/<id>")]
async fn read(mut db: Connection<Db>, id: i64) -> Option<Json<Note>> {
    sqlx::query!("SELECT * FROM notes WHERE id = $1", id)
        .fetch_one(&mut **db)
        .map_ok(|r| Json(Note {id: Some(r.id), title: r.title, message: r.message }))
        .await
        .ok()
}


pub fn stage() -> AdHoc {
    AdHoc::on_ignite("SQLx Stage", |rocket| async {
        rocket.attach(Db::init())
            .mount("/sqlx", routes![create, read])
    })
}
