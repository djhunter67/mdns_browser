use actix_web::{
    get,
    http::{header::ContentType, StatusCode},
    web, HttpResponse,
};
use askama::Template;
use mdns_scanner::{mdns_scan, Service, ServiceDetect};
use tracing::{error, info, instrument};

use crate::endpoints::templates::{ErrorPage, ScanResult};

#[instrument(
    name = "Serving main page",
    level = "debug",
    target = "web_app_bloodhound",
    fields(samples = 25, title = "Home")
)]
#[get("/scan_tech/{tech_to_scan}")]
pub async fn scan_tech(tech_to_scan: web::Path<String>) -> HttpResponse {
    info!("Initiating the scan for {}", tech_to_scan.clone());
    let service_detect: ServiceDetect = match tech_to_scan.clone().try_into() {
        Ok(service) => service,
        Err(err) => {
            error!("unable to convert the tech to scan into the enum required: {err:#?}");
            ServiceDetect::Http
        }
    };
    let scan = match mdns_scan(Some(service_detect), None) {
        Ok(res) => res,
        Err(err) => {
            error!("Scan result error or no results: {err}");
            vec![Service::error(err)]
        }
    };

    if scan
        .first()
        .expect("Scan result error or no results")
        .name()
        .contains("err")
    {
        let error_template = ErrorPage {
            title: &scan.first().expect("No error to parse").name(),
            code: u32::from(StatusCode::INTERNAL_SERVER_ERROR.as_u16()),
            error: "Scan Error",
            verbose_message: "No results found for the specified technology.",
        };
        return HttpResponse::InternalServerError()
            .content_type(ContentType::html())
            .body(
                error_template
                    .render()
                    .expect("Failed to render ERROR template"),
            );
    }
    let var_name = ScanResult {
        scan_domain: &tech_to_scan.into_inner(),
        results: scan,
    };
    let rendered = var_name.render().expect("Failed to render template");
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered)

    // let rendered = var_name.render().expect("Failed to render template");

    // HttpResponse::Ok()
    //     .content_type(ContentType::html())
    //     .body(rendered)
}
