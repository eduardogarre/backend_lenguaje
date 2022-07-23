# TRASTIENDA

Backend de [http://lenguajeñ.com](http://lenguajeñ.com), la página del [Lenguaje Ñ](https://github.com/eduardogarre/lenguaje)

## Detalles técnicos

He creado este proyecto con el lenguaje [Rust](https://www.rust-lang.org/) y el framework [Rocket](https://rocket.rs/).

El Backend sirve los archivos guardados dentro de la carpeta `./sitio/`, si ésta existe, a la vez que ofrece una api con 3 endpoints, accesibles a través de las rutas:
1) /api/v1/documento
1) /api/v1/sesión
1) /api/v1/usuario

## Comandos disponibles

### `cargo run`

Inicia el servidor en modo de desarrollo para mostrar el sitio.\
Abre la dirección [http://localhost:3000](http://localhost:3000) en el navegador para ver la página.

La página del navegador se recargará cuando guardes cambios en los archivos del proyecto.\
Además, en el terminal en el que hayas ejecutado `npm start` también podrás ver muchos errores de construcción.

### `cargo build`

Construye la versión de producción de la trastienda, y lo guarda en la carpeta `./target/release` con el nombre de `servidor`, listo para ser desplegado.