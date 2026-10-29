use rocket::{Rocket, Build, futures};
use rocket::fairing::{self, AdHoc};
use rocket_db_pools::{sqlx, Connection, Database};
use rocket::response::status::Created;
use rocket::serde::{Serialize, Deserialize, json::Json};
use futures::{TryFutureExt, TryStreamExt};

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

#[get("/")]
async fn list(mut db: Connection<Db>) -> Result<Json<Vec<Note>>> {
    let notes = sqlx::query!("SELECT * from notes")
        .fetch(&mut **db)
        .map_ok(|r| Note { id: Some(r.id), title: r.title, message: r.message })
        .try_collect::<Vec<_>>()
        .await?;

    Ok(Json(notes))
}

#[get("/<id>")]
async fn read(mut db: Connection<Db>, id: i64) -> Option<Json<Note>> {
    sqlx::query!("SELECT * FROM notes WHERE id = $1", id)
        .fetch_one(&mut **db)
        .map_ok(|r| Json(Note {id: Some(r.id), title: r.title, message: r.message }))
        .await
        .ok()
}

#[delete("/<id>")]
async fn delete(mut db: Connection<Db>, id: i64) -> Result<Option<()>> {
    let result = sqlx::query!("DELETE FROM notes WHERE id = $1", id)
        .execute(&mut **db)
        .await?;

    Ok((result.rows_affected() == 1).then_some(()))
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("db/sqlx/migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database with migrations: {}", e);
                Err(rocket)
            }
        }
        None => Err(rocket),
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("SQLx Stage", |rocket| async {
        rocket.attach(Db::init())
            .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
            .mount("/sqlx", routes![list, create, read, delete])
    })
}
