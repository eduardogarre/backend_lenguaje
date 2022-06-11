use rocket::http::Status;
use rocket::serde::json::{json, Json, Value};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket::State;

use super::id::Id;
use super::roles::Editor;
use super::usuarios::Usuario;

/**
 * Documentos
 */

// Inicio el contador en 1, porque el nodo 0 ya debería existir
// El nodo 0 es la raíz del árbol de documentos y debe existir para contener los hijos.
static mut CONTADOR_IDS: Id = 1;

unsafe fn lee_nuevo_id() -> Id {
    let id: Id = CONTADOR_IDS;
    CONTADOR_IDS = CONTADOR_IDS + 1;
    return id;
}

// Guardaré los documentos en este vector, respaldado por un archivo en el disco duro:
// "documentos.json", de este modo no preciso usar una BBDD.
pub type Documentos = Mutex<Vec<Documento>>;

async fn guarda_copia_documentos(documentos: String) {
    println!("¡Guardando documentos!");
    println!("{}", documentos);
    std::fs::write("documentos.json", documentos).unwrap();
}

// Estructuras con el contenido del documento y de la lista de todos los documentos.

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Documento {
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

// Puntos de entrada de la api de documentos:

#[get("/documentos", format = "json")]
async fn lee_documentos(lista: &State<Documentos>, _editor: Editor) -> Value {
    let lista = lista.lock().await;

    json!(*lista)
}

#[post("/documento", format = "json", data = "<documento>")]
async fn crea_documento(
    documento: Json<Documento>,
    lista: &State<Documentos>,
    _usuario: Usuario,
    _editor: Editor,
) -> Value {
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
    _usuario: Usuario,
    _editor: Editor,
) -> Option<Json<Documento>> {
    let mut lista = lista.lock().await;
    let doc = documento.into_inner();
    let i = lista.iter().position(|d| d.id == id).unwrap();
    (*lista)[i].padre = doc.padre;
    (*lista)[i].título = doc.título;
    (*lista)[i].contenido = doc.contenido;
    //No modifico la lista de hijos
    //(*lista)[i].hijos = doc.hijos;

    let j: String = serde_json::to_string_pretty(&(*lista)).unwrap();
    guarda_copia_documentos(j).await;

    return Some(Json((*lista)[i].clone()));
}

#[delete("/documento/<id>")]
async fn borra_documento(
    id: Id,
    lista: &State<Documentos>,
    _usuario: Usuario,
    _editor: Editor,
) -> Status {
    let mut lista = lista.lock().await;
    let i = lista.iter().position(|d| d.id == id).unwrap();
    let id_hijo = (*lista)[i].id;
    if ((*lista)[i].hijos.len() != 0) {
        return Status::Forbidden;
    }

    if (i != 0 && id_hijo != 0) {
        let id_padre = lista.iter().position(|d| d.id == lista[i].padre).unwrap();

        (*lista)[id_padre].hijos.retain(|&h| h != id_hijo);

        lista.remove(i);

        let j: String = serde_json::to_string_pretty(&(*lista)).unwrap();
        guarda_copia_documentos(j).await;

        return Status::Accepted;
    } else {
        return Status::Forbidden;
    }
}

pub fn prepara_estado_inicial() -> Documentos {
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

    let documentos: Documentos = match archivo {
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
                    Mutex::new(v)
                }
                None => {
                    // El vector estaba vacío o no era válido...
                    unsafe {
                        // inicio el contador de IDs...
                        CONTADOR_IDS = 1;
                    }
                    // y cargo el nodo raíz
                    Mutex::new(vec![doc_raíz])
                }
            }
        }
        Err(_e) => {
            // Si hay error al abrir los documentos, creo un nodo 0 inicial y establezco el contador de IDs a 0
            unsafe {
                CONTADOR_IDS = 1;
            }
            Mutex::new(vec![doc_raíz])
        }
    };

    return documentos;
}

pub fn rutas() -> Vec<rocket::Route> {
    routes![
        lee_documentos,
        crea_documento,
        lee_documento,
        cambia_documento,
        borra_documento
    ]
}
