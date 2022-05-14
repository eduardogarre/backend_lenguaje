#[macro_use]
extern crate rocket;

extern crate crypto;

use crypto::digest::Digest;
use crypto::sha3::Sha3;

use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest, Request};
use rocket::serde::json::{json, Json, Value};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket::State;

use rocket::http::{Cookie, CookieJar};

mod archivos;
mod cors;

/**
 * Acreditación
 */

// El tipo con el que represento un identificador
type Id = usize;

fn ofusca_clave(clave: &String) -> String {
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
 * Documentos
 */

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
    contenido: String,
    hijos: Vec<Id>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ListaDocumento {
    estado: String,
    documentos: Vec<Documento>,
}

async fn guarda_copia_documentos(documentos: String) {
    println!("¡Guardando documentos!");
    println!("{}", documentos);
    std::fs::write("documentos.json", documentos).unwrap();
}

impl Clone for Documento {
    fn clone(&self) -> Self {
        Documento {
            id: self.id.clone(),
            padre: self.padre.clone(),
            título: self.título.clone(),
            contenido: self.contenido.clone(),
            hijos: self.hijos.clone(),
        }
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

#[get("/documentos", format = "json")]
async fn lee_documentos(lista: &State<Documentos>) -> String {
    let lista = lista.lock().await;

    let j: String = serde_json::to_string_pretty(&(*lista)).unwrap();
    return j;
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

    let j: String = serde_json::to_string_pretty(&(*lista)).unwrap();
    guarda_copia_documentos(j).await;

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
        contenido: doc.contenido.clone(),
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
    (*lista)[i].contenido = doc.contenido;
    (*lista)[i].hijos = doc.hijos;

    let j: String = serde_json::to_string_pretty(&(*lista)).unwrap();
    guarda_copia_documentos(j).await;

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

        let j: String = serde_json::to_string_pretty(&(*lista)).unwrap();
        guarda_copia_documentos(j).await;

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
 * Monta todos los puntos de acceso
 */

fn stage() -> rocket::fairing::AdHoc {
    let clave: String = "1234".to_string();
    println!("La clave ofuscada es: {}", ofusca_clave(&clave));

    // Documento raíz, nodo 0
    let doc_raíz: Documento = Documento {
        // Nodo inicial
        id: 0,
        padre: 0,
        título: String::new(),
        contenido: String::new(),
        hijos: vec![],
    };

    // Intento cargar documentos previos
    let archivo = std::fs::read_to_string("documentos.json");

    let documentos: Vec<Documento> = match archivo {
        Ok(contenido) => {
            // Si he podido leer el archivo, intento procesarlo como JSON
            let v: Vec<Documento> = serde_json::from_str::<Vec<Documento>>(&contenido).unwrap();
            // Si me ha dejado procesarlo como JSON, intento encontrar el ID más grande
            let max_id_doc = v.iter().max_by_key(|doc| doc.id);
            match max_id_doc {
                // Si encuentra el ID más grande...
                Some(doc) => {
                    // establezco el contador de IDs...
                    unsafe {
                        CONTADOR_IDS = doc.id + 1;
                    }
                    // y devuelvo el vector de documentos
                    v
                }
                None => {
                    // El vector estaba vacío o no era válido...
                    unsafe {
                        // inicio el contador de IDs...
                        CONTADOR_IDS = 1;
                    }
                    // y cargo el nodo raíz
                    vec![doc_raíz]
                }
            }
        }
        Err(_e) => {
            // Si hay error al abrir los documentos, creo un nodo 0 inicial y establezco el contador de IDs a 0
            unsafe {
                CONTADOR_IDS = 1;
            }
            vec![doc_raíz]
        }
    };

    rocket::fairing::AdHoc::on_ignite("JSON", |rocket| async {
        rocket
            .mount("/", archivos::routes())
            .mount(
                "/api/v1/",
                routes![
                    secreto_accesible,
                    secreto_no_accesible,
                    gestiona_acceso,
                    cierra_sesión,
                    lee_documentos,
                    crea_documento,
                    lee_documento,
                    cambia_documento,
                    borra_documento
                ],
            )
            .register("/api/v1/", catchers![error_404, error_500])
            .manage(Documentos::new(documentos))
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build().attach(cors::CORS).attach(stage())
}
