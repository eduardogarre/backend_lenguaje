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

static mut CONTADOR_IDS: Id = 0;

unsafe fn lee_nuevo_id() -> Id {
    CONTADOR_IDS = CONTADOR_IDS + 1;
    let id: Id = CONTADOR_IDS;
    return id;
}

// Por ahora voy a guardar todos los documentos aquí, para no usar una BBDD.
type Documentos = Mutex<Vec<Documento>>;

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Documento {
    id: Id,
    título: String,
    párrafos: Vec<String>,
    hijos: Vec<Id>,
}

impl Clone for Documento {
    fn clone(&self) -> Self {
        Documento {
            id: self.id.clone(),
            título: self.título.clone(),
            párrafos: self.párrafos.clone(),
            hijos: self.hijos.clone(),
        }
    }
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
async fn crea_documento(documento: Json<Documento>, lista: &State<Documentos>) -> Value {
    let mut lista = lista.lock().await;
    let identificador: Id;

    unsafe {
        identificador = lee_nuevo_id();
    }

    let mut doc = documento.into_inner();
    doc.id = identificador;

    lista.push(doc);

    json!({ "estado": "ok", "id": Some(identificador) })
}

#[get("/documento/<id>", format = "json")]
async fn lee_documento(id: Id, lista: &State<Documentos>) -> Option<Json<Documento>> {
    let lista = lista.lock().await;
    let i = lista.iter().position(|d| d.id == id).unwrap();
    let doc: Documento = lista[i].clone();
    Some(Json(Documento {
        id: doc.id,
        título: doc.título.clone(),
        párrafos: doc.párrafos.clone(),
        hijos: doc.hijos.clone(),
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
            .manage(Documentos::new(vec![]))
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
