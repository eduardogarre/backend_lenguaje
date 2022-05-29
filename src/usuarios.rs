use rocket::http::Status;
use rocket::outcome::{try_outcome, IntoOutcome, Outcome::*};
use rocket::request::{self, FromRequest, Request};
use rocket::serde::json::{json, Json, Value};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::sync::Mutex;
use rocket::Config;
use rocket::State;

use super::id::Id;
use super::sesion;

/**
 * Usuarios
 */

// Estructuras con el usuario

#[derive(Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Usuario {
    pub id: Id,
    pub nombre: String,
    pub clave: String,
    pub roles: Vec<String>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Usuario {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Usuario, Self::Error> {
        // Accedo a la lista de sesiones
        let estado_sesiones = request
            .guard::<&State<sesion::SesionesActivas>>()
            .await
            .unwrap();
        let mutex_sesiones = estado_sesiones.lock().await;
        // Accedo a la cookie privada
        let cookie_sesión = request.cookies().get_private("sesión").unwrap();
        // Identifico la sesión guardada en la cookie privada
        let sesión_leída: String = cookie_sesión.value().to_string();
        // Leo el contenido de la sesión guardada en la cookie privada
        let contenido_sesión = (*mutex_sesiones).get(&sesión_leída).unwrap();

        // Accedo a la lista de usuarios
        let estado_usuarios = request.guard::<&State<Usuarios>>().await.unwrap();
        let mutex_usuarios = estado_usuarios.lock().await;
        // Busco el usuario con el identificador que se corresponda con el de la sesión activa
        let i = mutex_usuarios
            .iter()
            .position(|u| u.id == contenido_sesión.usuario)
            .unwrap();
        let usuario: Usuario = mutex_usuarios[i].clone();

        println!("Sesión leída: {}", sesión_leída);
        println!("id del usuario: {}", contenido_sesión.usuario.to_string());
        println!("nombre del usuario: {}", usuario.nombre);
        println!("clave del usuario: {}", usuario.clave);
        println!("caducidad: {:?}", contenido_sesión.caducidad);
        Some(usuario).or_forward(())
    }
}

pub struct Administrador {}

impl Clone for Administrador {
    fn clone(&self) -> Self {
        Administrador {}
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Administrador {
    type Error = std::convert::Infallible;

    async fn from_request(
        request: &'r Request<'_>,
    ) -> request::Outcome<Administrador, Self::Error> {
        // Accedo a la lista de sesiones
        let estado_sesiones = request
            .guard::<&State<sesion::SesionesActivas>>()
            .await
            .unwrap();
        let mutex_sesiones = estado_sesiones.lock().await;
        // Accedo a la cookie privada
        let cookie_sesión = request.cookies().get_private("sesión").unwrap();
        // Identifico la sesión guardada en la cookie privada
        let sesión_leída: String = cookie_sesión.value().to_string();
        // Leo el contenido de la sesión guardada en la cookie privada
        let contenido_sesión = (*mutex_sesiones).get(&sesión_leída).unwrap();

        // Accedo a la lista de usuarios
        let estado_usuarios = request.guard::<&State<Usuarios>>().await.unwrap();
        let mutex_usuarios = estado_usuarios.lock().await;
        // Busco el usuario con el identificador que se corresponda con el de la sesión activa
        let i = mutex_usuarios
            .iter()
            .position(|u| u.id == contenido_sesión.usuario)
            .unwrap();
        let usuario: Usuario = mutex_usuarios[i].clone();

        println!("Sesión leída: {}", sesión_leída);
        println!("id del usuario: {}", contenido_sesión.usuario.to_string());
        println!("nombre del usuario: {}", usuario.nombre);
        println!("clave del usuario: {}", usuario.clave);
        println!("roles del usuario: {:?}", usuario.roles);
        println!("caducidad: {:?}", contenido_sesión.caducidad);

        usuario
            .roles
            .into_iter()
            .find(|r| *r == "Administrador".to_string())
            .expect("El usuario no tiene el rol de administrador");

        Some(Administrador {}).or_forward(())
    }
}

pub struct Editor {}

impl Clone for Editor {
    fn clone(&self) -> Self {
        Editor {}
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Editor {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Editor, Self::Error> {
        // Accedo a la lista de sesiones
        let estado_sesiones = request
            .guard::<&State<sesion::SesionesActivas>>()
            .await
            .unwrap();
        let mutex_sesiones = estado_sesiones.lock().await;
        // Accedo a la cookie privada
        let cookie_sesión = request.cookies().get_private("sesión").unwrap();
        // Identifico la sesión guardada en la cookie privada
        let sesión_leída: String = cookie_sesión.value().to_string();
        // Leo el contenido de la sesión guardada en la cookie privada
        let contenido_sesión = (*mutex_sesiones).get(&sesión_leída).unwrap();

        // Accedo a la lista de usuarios
        let estado_usuarios = request.guard::<&State<Usuarios>>().await.unwrap();
        let mutex_usuarios = estado_usuarios.lock().await;
        // Busco el usuario con el identificador que se corresponda con el de la sesión activa
        let i = mutex_usuarios
            .iter()
            .position(|u| u.id == contenido_sesión.usuario)
            .unwrap();
        let usuario: Usuario = mutex_usuarios[i].clone();

        println!("Sesión leída: {}", sesión_leída);
        println!("id del usuario: {}", contenido_sesión.usuario.to_string());
        println!("nombre del usuario: {}", usuario.nombre);
        println!("clave del usuario: {}", usuario.clave);
        println!("roles del usuario: {:?}", usuario.roles);
        println!("caducidad: {:?}", contenido_sesión.caducidad);

        usuario
            .roles
            .into_iter()
            .find(|r| *r == "Editor".to_string())
            .expect("El usuario no tiene el rol de editor");

        Some(Editor {}).or_forward(())
    }
}

// Inicio el contador en 1, porque el nodo 0 ya debería existir
// El nodo 0 es el administrador
static mut CONTADOR_IDS: Id = 1;

unsafe fn lee_nuevo_id() -> Id {
    let id: Id = CONTADOR_IDS;
    CONTADOR_IDS = CONTADOR_IDS + 1;
    return id;
}

// Guardaré los usuarios en este vector, respaldado por un archivo en el disco duro:
// "usuarios.json", de este modo no preciso usar una BBDD.
pub type Usuarios = Mutex<Vec<Usuario>>;

async fn guarda_copia_usuarios(usuarios: String) {
    println!("¡Guardando usuarios!");
    println!("{}", usuarios);
    std::fs::write("usuarios.json", usuarios).unwrap();
}

// Estructuras con la lista de todos los usuarios.

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ListaUsuarios {
    estado: String,
    usuarios: Vec<Usuario>,
}

impl Clone for Usuario {
    fn clone(&self) -> Self {
        Usuario {
            id: self.id.clone(),
            nombre: self.nombre.clone(),
            clave: self.clave.clone(),
            roles: self.roles.clone(),
        }
    }
}

// Puntos de entrada de la api de usuarios:

#[get("/usuarios", format = "json")]
async fn lee_usuarios(lista: &State<Usuarios>, _administrador: Administrador) -> Value {
    let lista = lista.lock().await;

    json!(*lista)
}

#[post("/usuario", format = "json", data = "<usuario>")]
async fn crea_usuario(
    usuario: Json<Usuario>,
    lista: &State<Usuarios>,
    _usuario: Usuario,
    _administrador: Administrador,
) -> Value {
    let mut lista = lista.lock().await;
    let identificador: Id;

    unsafe {
        identificador = lee_nuevo_id();
    }

    let mut usu = usuario.into_inner();
    usu.id = identificador;

    lista.push(usu);

    let j: String = serde_json::to_string_pretty(&(*lista)).unwrap();
    guarda_copia_usuarios(j).await;

    json!({ "estado": "ok", "id": Some(identificador) })
}

#[get("/usuario/<id>", format = "json")]
async fn lee_usuario(id: Id, lista: &State<Usuarios>) -> Option<Json<Usuario>> {
    let lista = lista.lock().await;
    let i = lista.iter().position(|u| u.id == id).unwrap();
    let usu: Usuario = lista[i].clone();
    Some(Json(Usuario {
        id: usu.id,
        nombre: usu.nombre.clone(),
        clave: usu.clave.clone(),
        roles: usu.roles.clone(),
    }))
}

#[patch("/usuario/<id>", format = "json", data = "<usuario>")]
async fn cambia_usuario(
    id: Id,
    usuario: Json<Usuario>,
    lista: &State<Usuarios>,
    _usuario: Usuario,
) -> Option<Json<Usuario>> {
    let mut lista = lista.lock().await;
    let usu = usuario.into_inner();
    let i = lista.iter().position(|d| d.id == id).unwrap();
    (*lista)[i].nombre = usu.nombre;
    (*lista)[i].clave = usu.clave;
    //No modifico la lista de roles
    //(*lista)[i].roles = usu.roles;

    let j: String = serde_json::to_string_pretty(&(*lista)).unwrap();
    guarda_copia_usuarios(j).await;

    return Some(Json((*lista)[i].clone()));
}

#[delete("/usuario/<id>")]
async fn borra_usuario(id: Id, lista: &State<Usuarios>, _usuario: Usuario, _administrador: Administrador) -> Status {
    let mut lista = lista.lock().await;
    let i = lista.iter().position(|u| u.id == id).unwrap();

    if (i != 0) {
        lista.remove(i);

        let j: String = serde_json::to_string_pretty(&(*lista)).unwrap();
        guarda_copia_usuarios(j).await;

        return Status::Accepted;
    } else {
        return Status::Forbidden;
    }
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ConfigAdmin {
    admin: String,
    clave: String,
}

pub fn prepara_estado_inicial() -> Usuarios {
    let config_admin: ConfigAdmin = Config::figment().extract::<ConfigAdmin>().unwrap();
    // Usuario raíz, nodo 0
    let usu_raíz: Usuario = Usuario {
        // Nodo inicial
        id: 0,
        nombre: "Administrador".to_string(),
        clave: "1234".to_string(),
        roles: vec!["Administrador".to_string(), "Editor".to_string()],
    };

    // Intento cargar usuarios previos
    let archivo = std::fs::read_to_string("usuarios.json");

    let usuarios: Usuarios = match archivo {
        Ok(contenido) => {
            // Si he podido leer el archivo, intento procesarlo como JSON
            let v: Vec<Usuario> = serde_json::from_str::<Vec<Usuario>>(&contenido).unwrap();
            // Si me ha dejado procesarlo como JSON, intento encontrar el ID más grande
            let max_id_usu = v.iter().max_by_key(|usu| usu.id);
            match max_id_usu {
                // Si encuentra el ID más grande...
                Some(usu) => {
                    // establezco el contador de IDs...
                    unsafe {
                        CONTADOR_IDS = usu.id + 1;
                    }
                    // y devuelvo el vector de usuarios
                    Mutex::new(v)
                }
                None => {
                    // El vector estaba vacío o no era válido...
                    unsafe {
                        // inicio el contador de IDs...
                        CONTADOR_IDS = 1;
                    }
                    // y cargo el nodo raíz
                    Mutex::new(vec![usu_raíz])
                }
            }
        }
        Err(_e) => {
            // Si hay error al abrir los usuarios, creo un nodo 0 inicial y establezco el contador de IDs a 0
            unsafe {
                CONTADOR_IDS = 1;
            }
            Mutex::new(vec![usu_raíz])
        }
    };

    return usuarios;
}

pub fn rutas() -> Vec<rocket::Route> {
    routes![
        lee_usuarios,
        crea_usuario,
        lee_usuario,
        cambia_usuario,
        borra_usuario
    ]
}
