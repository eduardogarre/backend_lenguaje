extern crate crypto;

use crypto::digest::Digest;
use crypto::sha3::Sha3;

use rocket::http::{Cookie, CookieJar};
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest, Request};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};

use super::id::Id;

/**
 * Acreditación
 */

pub fn ofusca_clave(clave: &String) -> String {
    let mut olla = Sha3::sha3_512();
    olla.input_str(clave);
    olla.result_str()
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Acceso {
    usuario: String,
    clave: String,
}

#[derive(Debug)]
struct Usuario(Id);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Usuario {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Usuario, Self::Error> {
        request
            .cookies()
            .get_private("id_usuario")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(Usuario)
            .or_forward(())
    }
}

/**
 * Puntos de acceso de la API
 */

#[get("/secreto")]
fn secreto_accesible(usuario: Usuario) -> String {
    "secreto".to_string()
}

#[get("/secreto", rank = 2)]
fn secreto_no_accesible() -> String {
    "no tienes acceso".to_string()
}

#[post("/sesión", data = "<acceso>")]
fn gestiona_acceso(caja: &CookieJar<'_>, acceso: Json<Acceso>) -> Result<String, String> {
    if acceso.usuario == "Administrador" && acceso.clave == "1234" {
        caja.add_private(Cookie::new("id_usuario", 1.to_string()));
        Ok("Acceso concedido".to_string())
    } else {
        Err("Acceso denegado".to_string())
    }
}

#[delete("/sesión")]
fn cierra_sesión(caja: &CookieJar<'_>) -> String {
    caja.remove_private(Cookie::named("id_usuario"));
    "Sesión cerrada".to_string()
}

pub fn rutas() -> Vec<rocket::Route> {
    routes![
        secreto_accesible,
        secreto_no_accesible,
        gestiona_acceso,
        cierra_sesión
    ]
}
