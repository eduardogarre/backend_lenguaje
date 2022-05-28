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
    usuario: String,
    último_acceso: std::time::SystemTime,
    caducidad: std::time::SystemTime
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

#[derive(Debug)]
pub struct Usuario(Id);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Usuario {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Usuario, Self::Error> {
        //let estado_sesiones = try_outcome!(request.guard::<&State<SesionesActivas>>().await);
        let estado_sesiones = request.guard::<&State<SesionesActivas>>().await.unwrap();
        let mut mutex_sesiones = estado_sesiones.lock().await;

        let cookie_sesión = request.cookies().get_private("sesión").unwrap();
        let sesión_leída: String = cookie_sesión.value().to_string();
        let contenido_sesión = (*mutex_sesiones).get(&sesión_leída).unwrap();

        println!("Sesión leída: {}", sesión_leída);
        println!("usuario: {}", contenido_sesión.usuario);
        println!("último acceso: {:?}", contenido_sesión.último_acceso);
        println!("caducidad: {:?}", contenido_sesión.caducidad);

        request
            .cookies()
            .get_private("id_usuario")
            .and_then(|cookie| cookie.value().parse().ok())
            .map(Usuario)
            .or_forward(())
    }
}

fn crea_sesión(usuario: String) -> Sesión {
    let ahora: std::time::SystemTime = SystemTime::now();
    let caducidad: std::time::SystemTime = ahora.checked_add(Duration::from_secs(3600)).unwrap();
    let sesión = Sesión {
        usuario: usuario,
        último_acceso: ahora,
        caducidad: caducidad
    };
    return sesión;
}

fn crea_símbolo_sesión() -> String {
    let mut aleatorio = [0u8; 128];
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
