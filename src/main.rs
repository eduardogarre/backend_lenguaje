#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket::serde::json::{json, Value};

mod archivos;
mod cors;
mod documentos;
mod id;
mod sesion;
mod usuarios;

#[catch(401)]
fn error_401() -> Value {
    json!({
        "estado": "error",
        "código": 401,
        "mensaje": "No tienes permiso para acceder a este recurso."
    })
}

#[catch(403)]
fn error_403() -> Value {
    json!({
        "estado": "error",
        "código": 403,
        "mensaje": "Acción prohibida."
    })
}

#[catch(404)]
fn error_404() -> Value {
    json!({
        "estado": "error",
        "código": 404,
        "mensaje": "Recurso no encontrado."
    })
}

#[catch(500)]
fn error_500() -> Value {
    json!({
        "estado": "error",
        "código": 500,
        "mensaje": "Error interno."
    })
}

/**
 * Monta todos los puntos de acceso
 */

fn stage() -> rocket::fairing::AdHoc {
    let clave: String = "1234".to_string();
    println!("La clave ofuscada es: {}", sesion::ofusca_clave(&clave));

    rocket::fairing::AdHoc::on_ignite("JSON", |rocket| async {
        rocket
            .mount("/", archivos::rutas())
            .mount("/api/v1/", documentos::rutas())
            .mount("/api/v1/", sesion::rutas())
            .register(
                "/api/v1/",
                catchers![error_401, error_403, error_404, error_500],
            )
            .manage(documentos::prepara_estado_inicial())
            .manage(usuarios::prepara_estado_inicial())
            .manage(sesion::prepara_estado_inicial())
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build().attach(cors::CORS).attach(AdHoc::config::<sesion::ConfigAdmin>()).attach(stage())
}
