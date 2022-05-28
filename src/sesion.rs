extern crate base64;
extern crate crypto;
extern crate rand;

use crypto::digest::Digest;
use crypto::sha3::Sha3;

use rand::thread_rng;
use rand::Rng;

use rocket::http::{Cookie, CookieJar, Status};
use rocket::outcome::{try_outcome, IntoOutcome, Outcome::*};
use rocket::request::{self, FromRequest, Request};
use rocket::serde::json::{json, Json, Value};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket::Config;
use rocket::State;

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

use super::id::Id;
use super::usuarios::Usuario;

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

pub struct Sesión {
    pub usuario: Id,
    pub último_acceso: std::time::SystemTime,
    pub caducidad: std::time::SystemTime
}

pub type SesionesActivas = Mutex<HashMap<String, Sesión>>;

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

fn crea_sesión(usuario: String) -> Sesión {
    let ahora: std::time::SystemTime = SystemTime::now();
    let caducidad: std::time::SystemTime = ahora.checked_add(Duration::from_secs(3600)).unwrap();
    let sesión = Sesión {
        usuario: 0,
        último_acceso: ahora,
        caducidad: caducidad
    };
    return sesión;
}

fn crea_símbolo_sesión() -> String {
    let mut aleatorio = [0u8; 64];
    thread_rng().try_fill(&mut aleatorio[..]);
    format!("{}", base64::encode(&aleatorio))
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
async fn gestiona_acceso(caja: &CookieJar<'_>, acceso: Json<Acceso>, estado_sesiones: &State<SesionesActivas>) -> Result<Value, Status> {
    let config_admin: ConfigAdmin = Config::figment().extract::<ConfigAdmin>().unwrap();
    
    let mut mutex_sesiones = estado_sesiones.lock().await;

    if acceso.usuario == config_admin.admin && acceso.clave == config_admin.clave {

        caja.add_private(Cookie::new("id_usuario", 1.to_string()));

        let símbolo_sesión: String = crea_símbolo_sesión();
        let sesión: Sesión = crea_sesión(acceso.usuario.clone());
        (*mutex_sesiones).insert(símbolo_sesión.clone(), sesión);
        
        caja.add_private(Cookie::new("sesión", símbolo_sesión));

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
    caja.remove_private(Cookie::named("sesión"));
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
