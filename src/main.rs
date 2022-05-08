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

// Ya existe el nodo 0
static mut CONTADOR_IDS: Id = 1;

unsafe fn lee_nuevo_id() -> Id {
    let id: Id = CONTADOR_IDS;
    CONTADOR_IDS = CONTADOR_IDS + 1;
    return id;
}

// Por ahora voy a guardar todos los documentos aquí, para no usar una BBDD.
type Documentos = Mutex<Vec<Documento>>;

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Documento {
    id: Id,
    padre: Id,
    título: String,
    párrafos: Vec<String>,
    hijos: Vec<Id>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ListaDocumento {
    estado: String,
    documentos: Vec<Documento>,
}

impl Clone for Documento {
    fn clone(&self) -> Self {
        Documento {
            id: self.id.clone(),
            padre: self.padre.clone(),
            título: self.título.clone(),
            párrafos: self.párrafos.clone(),
            hijos: self.hijos.clone(),
        }
    }
}

#[get("/documentos")]
async fn lee_documentos(lista: &State<Documentos>) -> Option<Json<ListaDocumento>> {
    let lista = lista.lock().await;
    let l = (*lista).clone();

    Some(Json(ListaDocumento {
        estado: "ok".to_string(),
        documentos: l,
    }))
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

    let id_padre = lista.iter().position(|d| d.id == doc.padre).unwrap();
    lista[id_padre].hijos.push(identificador);

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
        padre: doc.padre,
        título: doc.título.clone(),
        párrafos: doc.párrafos.clone(),
        hijos: doc.hijos.clone(),
    }))
}

#[patch("/documento/<id>", format = "json", data = "<documento>")]
async fn cambia_documento(
    id: Id,
    documento: Json<Documento>,
    lista: &State<Documentos>,
) -> Option<Json<Documento>> {
    let mut lista = lista.lock().await;
    let doc = documento.into_inner();
    let i = lista.iter().position(|d| d.id == id).unwrap();
    (*lista)[i].padre = doc.padre;
    (*lista)[i].título = doc.título;
    (*lista)[i].párrafos = doc.párrafos;
    (*lista)[i].hijos = doc.hijos;

    return Some(Json((*lista)[i].clone()));
}

#[delete("/documento/<id>")]
async fn borra_documento(id: Id, lista: &State<Documentos>) -> Value {
    let mut lista = lista.lock().await;
    let i = lista.iter().position(|d| d.id == id).unwrap();
    if ((*lista)[i].hijos.len() != 0) {
        return json!({
            "estado": "error",
            "código": 405,
            "mensaje": "Método no permitido, no puedes borrar un nodo que tenga hijos."
        });
    }

    if (i != 0) {
        let id_padre = lista.iter().position(|d| d.id == lista[i].padre).unwrap();

        (*lista)[id_padre].hijos = lista[id_padre]
            .hijos
            .drain(..)
            .filter(|id_hijo| *id_hijo != i)
            .collect();

        lista.remove(i);
        return json!({
            "estado": "ok",
            "código": 200
        });
    } else {
        return json!({
            "estado": "error",
            "código": 405,
            "mensaje": "Método no permitido, no puedes borrar el nodo raíz."
        });
    }
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
            .register("/api/v1/", catchers![error_404, error_500])
            .mount(
                "/",
                routes![
                    archivo_raiz,
                    archivo_index_htm,
                    archivos,
                    archivos_predeterminado
                ],
            )
            .manage(Documentos::new(vec![Documento {
                // Nodo inicial
                id: 0,
                padre: 0,
                título: String::new(),
                párrafos: vec![],
                hijos: vec![],
            }]))
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
