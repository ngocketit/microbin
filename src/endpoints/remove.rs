use actix_web::{get, web, HttpResponse};

use crate::args::ARGS;
use crate::endpoints::errors::ErrorTemplate;
use crate::pasta::PastaFile;
use crate::util::misc::{get_pasta_id, remove_expired};
use crate::AppState;
use askama::Template;
use std::fs;

#[get("/remove/{id}")]
pub async fn remove(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    if ARGS.readonly {
        return HttpResponse::Found()
            .append_header(("Location", format!("{}/", ARGS.public_path)))
            .finish();
    }

    let mut pastas = data.pastas.lock().unwrap();

    let id = get_pasta_id(id);

    for (i, pasta) in pastas.iter().enumerate() {
        if pasta.id == id {
            // remove the file itself
            if let Some(PastaFile { name, .. }) = &pasta.file {
                if fs::remove_file(format!(
                    "./pasta_data/public/{}/{}",
                    pasta.id_as_animals(),
                    name
                ))
                .is_err()
                {
                    log::error!("Failed to delete file {}!", name)
                }

                // and remove the containing directory
                if fs::remove_dir(format!("./pasta_data/public/{}/", pasta.id_as_animals()))
                    .is_err()
                {
                    log::error!("Failed to delete directory {}!", name)
                }
            }
            // remove it from in-memory pasta list
            pastas.remove(i);
            return HttpResponse::Found()
                .append_header(("Location", format!("{}/pastalist", ARGS.public_path)))
                .finish();
        }
    }

    remove_expired(&mut pastas);

    HttpResponse::Ok()
        .content_type("text/html")
        .body(ErrorTemplate { args: &ARGS }.render().unwrap())
}
