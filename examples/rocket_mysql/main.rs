#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_contrib;

use rocket::http::Status;
use rocket::request::Form;
use rocket::State;
use rocket_contrib::databases::mysql;
use rocket_contrib::json::Json;

use serde::Deserialize;
use serde::Serialize;

use toql::derive::Toql;

use toql::query::Query;
use toql::error::ToqlError;
use toql::sql_mapper::SqlMapper;
use toql::sql_mapper::SqlMapperCache;
use toql::Result;

use toql::mysql::delete_one;
use toql::mysql::insert_one;
use toql::mysql::load_one;
use toql::mysql::update_one;

use toql::rocket::mysql::load_many;
use toql::rocket::Counted;
use toql::rocket::ToqlQuery;

// Here is our struct
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Toql)]
pub struct User {
    #[toql(delup_key, skip_inup)]       // Use id for delete/update key, don't insert/update (because column auto increments)
    #[serde(skip_deserializing)]
    pub id: u64,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub username: Option<String>,
}

// Here come the crud functions
#[delete("/<id>")]
pub fn delete<'a>(id: u64, conn: ExampleDbConnection) -> Result<Status> {
    let ExampleDbConnection(mut c) = conn;
    let u = User {
        id: id,
        ..Default::default()
    };

    let _affected_rows = delete_one(&u, &mut c)?;
    if _affected_rows == 0 {
        return Err(ToqlError::NotFound);
    }
    Ok(Status::NoContent)
}

#[put("/<id>", data = "<user>")]
pub fn update(
    id: u64,
    mut user: Json<User>,
    mappers: State<SqlMapperCache>,
    conn: ExampleDbConnection,
) -> Result<Json<User>> {
    let ExampleDbConnection(mut c) = conn;
    user.id = id;
    let _affected_rows = update_one(&user.into_inner(), &mut c)?;

    let q = Query::wildcard().and(User::fields().id().eq(id));
    let u = load_one::<User>(&q, &mappers, &mut c)?;
    Ok(Json(u))
}

#[post("/", data = "<user>")] // format = "application/json",
pub fn create<'a>(
    user: Json<User>,
    mappers: State<SqlMapperCache>,
    conn: ExampleDbConnection,
) -> Result<Json<User>> {
    let ExampleDbConnection(mut c) = conn;
    let last_id = insert_one(&user.into_inner(), &mut c)?;

    let q = Query::wildcard().and(User::fields().id().eq(last_id));
    let u = load_one::<User>(&q, &mappers, &mut c)?;
    Ok(Json(u))
}

#[get("/<id>")]
pub fn get(
    id: u64,
    mappers: State<SqlMapperCache>,
    conn: ExampleDbConnection,
) -> Result<Json<User>> {
    let ExampleDbConnection(mut c) = conn;

    let query = Query::wildcard().and(User::fields().id().eq(id));
    let u = load_one::<User>(&query, &mappers, &mut c)?;
    Ok(Json(u))
}

#[get("/?<toql..>")]
pub fn query(
    mappers: State<SqlMapperCache>,
    conn: ExampleDbConnection,
    toql: Form<ToqlQuery>,
) -> Result<Counted<Json<Vec<User>>>> {
    let ExampleDbConnection(mut c) = conn;

    let r = load_many::<User>(&toql, &mappers, &mut c)?;
    Ok(Counted(Json(r.0), r.1))
}

// The database connection
#[database("example_db")]
pub struct ExampleDbConnection(mysql::Conn);

// Main to startup the server
fn main() {
    println!("------------------------------------------");
    println!("Full Toql CRUD example with Rocket / MySql");
    println!("------------------------------------------");
    println!("This example assumes that you have a MySQL Server");
    println!("running with a database `example_db`");
    println!("Run the following SQL to create the table `User`");
    println!("CREATE TABLE `User` (`id` int(11) NOT NULL AUTO_INCREMENT,`username` varchar(200) NOT NULL, PRIMARY KEY (id))");
    println!("------------------------------------------------------------------------------------------------------------");
    println!("Start the server with ");
    println!("ROCKET_DATABASES={{example_db={{url=mysql://USER:PASS@localhost:3306/example_db}}}} cargo +nightly run --example crud_rocket_mysql");
    println!("----------------------------------------------------------------------------------------------------------------------");
    println!("Create a user with `curl localhost:8000/user -X POST -d '{{\"username\":\"Peter\"}}'`");
    println!("Update a user with `curl localhost:8000/user/ID -X PUT -d '{{\"username\":\"Susan\"}}'`");
    println!("Get a single user with `curl localhost:8000/user/ID`");
    println!("Get all users with `curl localhost:8000/user`");
    println!("Get only id from users in descending order `curl localhost:8000/user?query=-id`");
    println!("Delete a user with `curl -X DELETE localhost:8000/user/ID`");
    println!("--------------------------");
    
    let mut mappers = SqlMapperCache::new();
    SqlMapper::insert_new_mapper::<User>(&mut mappers);

    rocket::ignite()
        .manage(mappers)
        .attach(ExampleDbConnection::fairing())
        .mount("/user", routes![get, query, create, update, delete])
        .launch();
}
