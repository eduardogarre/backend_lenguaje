#[macro_use]
extern crate rocket;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::fs::NamedFile;
use rocket::http::Header;
use rocket::{Request, Response};
use std::path::{Path, PathBuf};

// Configura CORS

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

/**
 * Puntos de acceso de la API
 */

#[get("/documentos")]
fn lee_documentos() -> &'static str {
    "{
        \"arr\": [
            \"Lista\",
            \"de\",
            \"documentos\"
            ]
    }"
}

#[post("/documento")]
fn crea_documento() -> &'static str {
    "Crea un nuevo documento"
}

#[get("/documento/<id>")]
fn lee_documento(id: u64) -> std::string::String {
    return format!("Lee el documento {}", id);
}

#[patch("/documento/<id>")]
fn cambia_documento(id: u64) -> std::string::String {
    return format!("Cambia el documento {}", id);
}

#[delete("/documento/<id>")]
fn borra_documento(id: u64) -> std::string::String {
    return format!("Borra el documento {}", id);
}

/**
 * Puntos de acceso para los archivos estáticos
 */

#[get("/", rank = 2)]
async fn archivo_raiz() -> Option<NamedFile> {
    NamedFile::open(Path::new("build/").join("index.html"))
        .await
        .ok()
}

#[get("/index.htm", rank = 2)]
async fn archivo_index_htm() -> Option<NamedFile> {
    NamedFile::open(Path::new("build/").join("index.html"))
        .await
        .ok()
}

#[get("/<archivo..>", rank = 3)]
async fn archivos(archivo: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("build/").join(archivo))
        .await
        .ok()
}

#[get("/<archivo..>", rank = 4)]
async fn archivos_predeterminado(archivo: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("build/").join("index.html"))
        .await
        .ok()
}

/**
 * Monta todos los puntos de acceso
 */

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(CORS)
        .mount(
            "/api/v1/",
            routes![
                lee_documentos,
                crea_documento,
                lee_documento,
                cambia_documento,
                borra_documento
            ],
        )
        .mount(
            "/",
            routes![
                archivo_raiz,
                archivo_index_htm,
                archivos,
                archivos_predeterminado
            ],
        )
}

/*

GET                 /api/v1/tutoriales
POST                /api/v1/tutorial
GET/PATCH/DELETE   /api/v1/tutorial/:id

GET                 /api/v1/referencias
POST                /api/v1/referencia
GET/PATCH/DELETE   /api/v1/referencia/:id

*/
