#[macro_use] extern crate rocket;

mod sqlx;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().attach(sqlx::stage()).mount("/", routes![index])
}
