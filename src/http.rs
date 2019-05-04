use actix::prelude::*;
use actix_web::{App, AsyncResponder, Form, FromRequest, FutureResponse, HttpRequest, HttpResponse, Responder, server};
use actix_web::http::Method;
use actix_web::middleware::Logger;
use futures::Future;
use serde::Deserialize;

use crate::config::Config;
use crate::db::{CreateUser, Database, DemoteUser, PromoteUser, ResetPassword};

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

#[derive(Deserialize)]
struct PromoteUserFormData {
    charname: String,
    secret: String,
}

fn promote_user(req: &HttpRequest<State>) -> FutureResponse<HttpResponse> {
    let req = req.clone();
    Form::<PromoteUserFormData>::extract(&req)
        .and_then(move |data| {
            let result: Box<dyn futures::Future<Item=_, Error=_>> = if data.secret == req.state().config.promote_secret {
                Box::new(req.state().db.send(PromoteUser {
                    charname: data.charname.clone(),
                })
                    .from_err()
                    .and_then(|res| {
                        match res {
                            Ok(_) => Ok(HttpResponse::Ok().body("Successfully promoted user")),
                            Err(e) => {
                                eprintln!("Error promoting user:\n    {:#?}", e);
                                Ok(HttpResponse::InternalServerError().body("Failed to promote user"))
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

#[derive(Deserialize)]
struct DemoteUserFormData {
    charname: String,
    secret: String,
}

fn demote_user(req: &HttpRequest<State>) -> FutureResponse<HttpResponse> {
    let req = req.clone();
    Form::<DemoteUserFormData>::extract(&req)
        .and_then(move |data| {
            let result: Box<dyn futures::Future<Item=_, Error=_>> = if data.secret == req.state().config.demote_secret {
                Box::new(req.state().db.send(DemoteUser {
                    charname: data.charname.clone(),
                })
                    .from_err()
                    .and_then(|res| {
                        match res {
                            Ok(_) => Ok(HttpResponse::Ok().body("Successfully demoted user")),
                            Err(e) => {
                                eprintln!("Error demoting user:\n    {:#?}", e);
                                Ok(HttpResponse::InternalServerError().body("Failed to demote user"))
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

#[derive(Deserialize)]
struct ResetPasswordFormData {
    username: String,
    password: String,
    secret: String,
}

fn reset_password(req: &HttpRequest<State>) -> FutureResponse<HttpResponse> {
    let req = req.clone();
    Form::<ResetPasswordFormData>::extract(&req)
        .and_then(move |data| {
            let result: Box<dyn futures::Future<Item=_, Error=_>> = if data.secret == req.state().config.resetpw_secret {
                Box::new(req.state().db.send(ResetPassword {
                    username: data.username.clone(),
                    password: data.password.clone(),
                })
                    .from_err()
                    .and_then(|res| {
                        match res {
                            Ok(_) => Ok(HttpResponse::Ok().body("Successfully reset password")),
                            Err(e) => {
                                eprintln!("Error resetting password:\n    {:#?}", e);
                                Ok(HttpResponse::InternalServerError().body("Failed to reset password"))
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

fn manifest(_req: &HttpRequest<State>) -> impl Responder {
    use std::{io, fs, env};
    let mut path = env::current_exe().unwrap();
    path.set_file_name("manifest.xml");
    match fs::read(path) {
        Ok(data) => HttpResponse::Ok()
            .content_type("text/xml")
            .body(data),
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => HttpResponse::NotFound().into(),
            _ => HttpResponse::InternalServerError().body("Error reading manifest.xml")
        }
    }
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
            .middleware(Logger::new("%a %t \"%r\" %s %b %T"))
            .resource("/", |r| r.f(index))
            .resource("/create-user", |r| r.method(Method::POST).a(create_user))
            .resource("/promote-user", |r| r.method(Method::POST).a(promote_user))
            .resource("/demote-user", |r| r.method(Method::POST).a(demote_user))
            .resource("/reset-password", |r| r.method(Method::POST).a(reset_password))
            .resource("/manifest.xml", |r| r.f(manifest))
    }

    pub fn start(self) {
        let port = self.config.port;
        server::new(move || self.app())
            .bind(("0.0.0.0", port))
            .expect(&format!("Can not bind to port {}", port))
            .start();
    }
}
