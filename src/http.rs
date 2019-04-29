use actix::prelude::*;
use actix_web::{App, AsyncResponder, Form, FromRequest, FutureResponse, HttpRequest, HttpResponse, Responder, server};
use actix_web::http::Method;
use futures::Future;
use serde::Deserialize;

use crate::config::Config;
use crate::db::{CreateUser, Database};

struct State {
    config: Config,
    db: Addr<Database>,
}

#[derive(Clone)]
pub struct Server {
    config: Config,
    db: Addr<Database>,
}

fn index(_req: &HttpRequest<State>) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("../assets/index.html"))
}

#[derive(Deserialize)]
struct CreateUserFormData {
    username: String,
    password: String,
    secret: String,
}

fn create_user(req: &HttpRequest<State>) -> FutureResponse<HttpResponse> {
    let req = req.clone();
    Form::<CreateUserFormData>::extract(&req)
        .and_then(move |data| {
            let result: Box<dyn futures::Future<Item=_, Error=_>> = if data.secret == req.state().config.register_secret {
                Box::new(req.state().db.send(CreateUser {
                    username: data.username.clone(),
                    password: data.password.clone(),
                })
                    .from_err()
                    .and_then(|res| {
                        match res {
                            Ok(_) => Ok(HttpResponse::Ok().body("Successfully created user")),
                            Err(e) => {
                                eprintln!("Error creating user:\n    {:#?}", e);
                                Ok(HttpResponse::InternalServerError().body("Failed to create user"))
                            }
                        }
                    }))
            } else {
                Box::new(futures::future::ok(HttpResponse::Unauthorized().body("Wrong secret!")))
            };
            result
        })
        .responder()
}

impl Server {
    pub fn new(config: Config, db: Addr<Database>) -> Server {
        Server {
            config,
            db,
        }
    }

    fn app(&self) -> App<State> {
        App::with_state(State {
            config: self.config.clone(),
            db: self.db.clone(),
        })
            .resource("/", |r| r.f(index))
            .resource("/create-user", |r| r.method(Method::POST).a(create_user))
    }

    pub fn start(self) {
        let port = self.config.port;
        server::new(move || self.app())
            .bind(("127.0.0.1", port))
            .expect(&format!("Can not bind to port {}", port))
            .start();
    }
}
