use std::borrow::Cow;
use std::path::{Path, PathBuf};

#[macro_use]
extern crate rocket;

use rocket::fairing::{Fairing, Info, Kind};
use rocket::fs::NamedFile;
use rocket::http::Header;
use rocket::serde::json::{json, Json, Value};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket::State;
use rocket::{Request, Response};

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

// El tipo con el que represento el identificador de un mensaje
type Id = usize;

// Por ahora voy a guardar todos los documentos aquí, para no usar una BBDD.
type ListaDocumentos = Mutex<Vec<String>>;
type Documentos<'r> = &'r State<ListaDocumentos>;

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Documento<'r> {
    id: Option<Id>,
    contenido: Cow<'r, str>,
}

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

#[post("/documento", format = "json", data = "<documento>")]
async fn crea_documento(documento: Json<Documento<'_>>, lista: Documentos<'_>) -> Value {
    let mut lista = lista.lock().await;
    let id = lista.len();
    lista.push(documento.contenido.to_string());
    json!({ "estado": "ok", "id": id })
}

#[get("/documento/<id>", format = "json")]
async fn lee_documento(id: Id, list: Documentos<'_>) -> Option<Json<Documento<'_>>> {
    let list = list.lock().await;

    Some(Json(Documento {
        id: Some(id),
        contenido: list.get(id)?.to_string().into(),
    }))
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

fn stage() -> rocket::fairing::AdHoc {
    rocket::fairing::AdHoc::on_ignite("JSON", |rocket| async {
        rocket
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
            .manage(ListaDocumentos::new(vec![]))
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build().attach(CORS).attach(stage())
}

/*

GET                 /api/v1/tutoriales
POST                /api/v1/tutorial
GET/PATCH/DELETE   /api/v1/tutorial/:id

GET                 /api/v1/referencias
POST                /api/v1/referencia
GET/PATCH/DELETE   /api/v1/referencia/:id

*/
