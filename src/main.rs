use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, guard};
use std::sync::Mutex;
mod config;

// This struct represents state
struct AppState {
    app_name: String,
}

struct AppStateWithCounter {
    counter: Mutex<i32>, // <- Mutex is necessary to mutate safely across threads
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

async fn index(name: web::Data<AppState>, counter: web::Data<AppStateWithCounter>) -> String {
    let app_name = &name.app_name;
    let mut counter = counter.counter.lock().unwrap(); // <- get counter's MutexGuared
    *counter += 1; // <- access counter inside MutexGuard

    format!("Hi {app_name}!\n\tRequest number: {counter}") // <- response with app_name
}

#[get("/show")]
async fn show_users() -> impl Responder {
    HttpResponse::Ok().body("List of users...")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let scope = web::scope("/users").service(show_users);
    App::new().service(scope);
    
    // Note: web::Data created _outside_ HttpServer::new closure
    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });
    
    HttpServer::new(move || {
        // move counter into the closure
        App::new()
            .configure(config::config)
            .service(
                web::scope("/app")
                    .route("/index.html", web::get().to(index))
                    .app_data(counter.clone()) // <- register the created data
                    .app_data(web::Data::new(AppState {
                        app_name: String::from("Actix Web"),
                    }))
            )
            .service(hello)
            .service(
                web::scope("/")
                    .configure(config::scoped_config)
                    .guard(guard::Header("Host", "www.rust-lang.org"))
                    .route("", web::to(|| async { HttpResponse::Ok().body("www") }))
            )
            .service(
                web::scope("/")
                    .guard(guard::Header("Host", "users.rust-lang.org"))
                    .route("", web::to(|| async { HttpResponse::Ok().body("user") }))
            )
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
