#[macro_use] extern crate rocket;

mod sqlx;
mod diesel_postgres;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(sqlx::stage())
        .attach(diesel_postgres::stage())
        .mount("/", routes![index])
}
