#[macro_use]
extern crate diesel;

use actix_web::{delete, error, get, middleware, patch, post, web, App, HttpResponse, HttpServer, Responder};
use diesel::{prelude::*, r2d2};
use dotenvy;
mod models;
mod actions;
mod schema;


type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

#[get("/tasks")]
async fn get_tasks(pool: web::Data<DbPool>) -> actix_web::Result<impl Responder>{

    let tasks = web::block(move || {
        let mut conn: r2d2::PooledConnection<r2d2::ConnectionManager<PgConnection>> = pool.get()?;
        actions::find_all_tasks(&mut conn)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    
    Ok(HttpResponse::Ok().json(tasks))
}

#[post("/task_create")]
async fn create_task(pool: web::Data<DbPool>, form: web::Json<models::NewTask>) -> actix_web::Result<impl Responder> {
    let new_task = web::block(move || {
        let mut conn = pool.get()?;
        actions::create_task(&mut conn, &form.title, form.description.as_deref(), form.completed)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    Ok(HttpResponse::Created().json(new_task))
}

#[get("/task/{id}")]
async fn get_task_by_id(pool: web::Data<DbPool>, id: web::Path<i32>) -> actix_web::Result<impl Responder> {
    let task = web::block(move || {
        let mut conn = pool.get()?;
        actions::find_task_by_id(&mut conn, *id)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    match task {
        Some(task) => Ok(HttpResponse::Ok().json(task)),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

#[patch("/task/{id}")]
async fn update_task(pool: web::Data<DbPool>, id: web::Path<i32>, form: web::Json<models::UpdateTask>) -> actix_web::Result<impl Responder> {
    let updated_task = web::block(move || {
        let mut conn = pool.get()?;
        actions::update_task(&mut conn, *id, form.title.as_deref(), form.description.as_deref(), form.completed)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    match updated_task {
        Some(task) => Ok(HttpResponse::Ok().json(task)),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

#[delete("/task/{id}")]
async fn delete_task(pool: web::Data<DbPool>, id: web::Path<i32>) -> actix_web::Result<impl Responder> {
    let deleted_task = web::block(move || {
        let mut conn = pool.get()?;
        actions::delete_task(&mut conn, *id)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    match deleted_task {
        Some(task) => Ok(HttpResponse::Ok().json(task)),
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let pool = initialize_db_pool();

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            // add DB pool handle to app data; enables use of `web::Data<DbPool>` extractor
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .service(get_tasks)
            .service(create_task)
            .service(get_task_by_id)
            .service(update_task)
            .service(delete_task)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}


fn initialize_db_pool() -> DbPool {
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let manager = r2d2::ConnectionManager::<PgConnection>::new(conn_spec);
    r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid path to SQLite DB file")
}
