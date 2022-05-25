extern crate crypto;

use crypto::digest::Digest;
use crypto::sha3::Sha3;

use rocket::Config;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest, Request};
use rocket::serde::json::{json, Json, Value};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use std::collections::HashMap;

use super::id::Id;

pub type SesionesActivas = Mutex<HashMap<String, String>>;

/**
 * Acreditación
 */

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ConfigAdmin {
    admin: String,
    clave: String,
}

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

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct RespuestaJson {
    mensaje: String,
}

#[derive(Debug)]
pub struct Usuario(Id);

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

#[get("/sesión")]
fn secreto_accesible(_usuario: Usuario) -> Value {
    json!(RespuestaJson {
        mensaje: "Secreto muy valioso.".to_string()
    })
}

#[get("/sesión", rank = 2)]
fn secreto_no_accesible() -> Status {
    Status::Unauthorized
}

#[post("/sesión", data = "<acceso>")]
fn gestiona_acceso(caja: &CookieJar<'_>, acceso: Json<Acceso>) -> Result<Value, Status> {
    
    let config_admin: ConfigAdmin = Config::figment().extract::<ConfigAdmin>().unwrap();

    if acceso.usuario == config_admin.admin && acceso.clave == config_admin.clave {
        caja.add_private(Cookie::new("id_usuario", 1.to_string()));
        Ok(json!(RespuestaJson {
            mensaje: "Acceso concedido.".to_string()
        }))
    } else {
        Err(Status::Unauthorized)
    }
}

#[delete("/sesión")]
fn cierra_sesión(caja: &CookieJar<'_>) -> Value {
    caja.remove_private(Cookie::named("id_usuario"));
    json!(RespuestaJson {
        mensaje: "Sesión cerrada.".to_string()
    })
}

pub fn prepara_estado_inicial() -> SesionesActivas {
    let dic_vacío = HashMap::new();
    Mutex::new(dic_vacío)
}

pub fn rutas() -> Vec<rocket::Route> {
    routes![
        secreto_accesible,
        secreto_no_accesible,
        gestiona_acceso,
        cierra_sesión
    ]
}
