use crate::args::{Args, ARGS};
use crate::endpoints::errors::ErrorTemplate;
use crate::pasta::Pasta;
use crate::util::misc::{self, get_pasta_id, remove_expired};
use crate::AppState;
use actix_web::{get, web, HttpResponse};
use askama::Template;

#[derive(Template)]
#[template(path = "qr.html", escape = "none")]
struct QRTemplate<'a> {
    qr: &'a String,
    pasta: &'a Pasta,
    args: &'a Args,
}

#[get("/qr/{id}")]
pub async fn getqr(data: web::Data<AppState>, id: web::Path<String>) -> HttpResponse {
    // get access to the pasta collection
    let mut pastas = data.pastas.lock().unwrap();

    let id = get_pasta_id(id);

    // remove expired pastas (including this one if needed)
    remove_expired(&mut pastas);

    match pastas.iter().enumerate().find(|(_, pasta)| pasta.id == id) {
        Some((index, _)) => {
            // generate the QR code as an SVG - if its a file or text pastas, this will point to the /pasta endpoint, otherwise to the /url endpoint, essentially directly taking the user to the url stored in the pasta
            let svg: String = match pastas[index].pasta_type.as_str() {
                "url" => {
                    misc::string_to_qr_svg(format!("{}/url/{}", &ARGS.public_path, &id).as_str())
                }
                _ => {
                    misc::string_to_qr_svg(format!("{}/pasta/{}", &ARGS.public_path, &id).as_str())
                }
            };

            // serve qr code in template
            HttpResponse::Ok().content_type("text/html").body(
                QRTemplate {
                    qr: &svg,
                    pasta: &pastas[index],
                    args: &ARGS,
                }
                .render()
                .unwrap(),
            )
        }
        _ => HttpResponse::Ok()
            .content_type("text/html")
            .body(ErrorTemplate { args: &ARGS }.render().unwrap()),
    }
}
