#[macro_use]
extern crate rocket;

use rocket::serde::json::{json, Value};

mod archivos;
mod cors;
mod documentos;
mod id;
mod sesion;

use documentos::{prepara_documentos, Documentos};

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

    let documentos: Documentos = prepara_documentos();

    rocket::fairing::AdHoc::on_ignite("JSON", |rocket| async {
        rocket
            .mount("/", archivos::rutas())
            .mount("/api/v1/", documentos::rutas())
            .mount("/api/v1/", sesion::rutas())
            .register("/api/v1/", catchers![error_404, error_500])
            .manage(documentos)
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build().attach(cors::CORS).attach(stage())
}
