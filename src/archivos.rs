use rocket::fs::NamedFile;
use std::path::{Path, PathBuf};

/**
 * Puntos de acceso para los archivos estáticos
 */

#[get("/", rank = 2)]
async fn archivo_raiz() -> Option<NamedFile> {
    NamedFile::open(Path::new("sitio/").join("index.html"))
        .await
        .ok()
}

#[get("/index.htm", rank = 2)]
async fn archivo_index_htm() -> Option<NamedFile> {
    NamedFile::open(Path::new("sitio/").join("index.html"))
        .await
        .ok()
}

#[get("/<archivo..>", rank = 3)]
async fn archivos(archivo: PathBuf) -> Option<NamedFile> {
    let arch = NamedFile::open(Path::new("sitio/").join(archivo)).await;
    let resultado = match arch {
        Ok(a) => a,
        Err(_e) => {
            return NamedFile::open(Path::new("sitio/").join("index.html"))
                .await
                .ok()
        }
    };
    return Some(resultado);
}

pub fn rutas() -> Vec<rocket::Route> {
    routes![archivo_raiz, archivo_index_htm, archivos]
}
