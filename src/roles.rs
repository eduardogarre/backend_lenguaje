use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest, Request};
use rocket::State;

use super::sesion;
use super::usuarios::{Usuario, Usuarios};

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
