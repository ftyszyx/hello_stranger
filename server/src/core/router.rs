use salvo::prelude::*;
use salvo::cors::{Cors, AllowOrigin, AllowHeaders};
use salvo::http::Method;
use crate::core::app::AppState;
use salvo_oapi::{OpenApi, SecurityScheme};
use salvo_oapi::security::{Http, HttpAuthScheme};
use crate::apis::*;

#[handler]
async fn hello(res: &mut Response) {
    res.render(Text::Plain("Hello, world!"));
}

pub fn create_router(app_state: AppState) -> Service {
    // let admin_routes = Router::with_path("/api/admin")
    //     // .hoop(middleware::auth)
    //     // .hoop(middleware::error_handler)

    let cors = Cors::new()
    .allow_origin(AllowOrigin::any())
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
    .allow_headers(AllowHeaders::any()).into_handler();
    let mut router=Router::new()
        .hoop(affix_state::inject(app_state))
        .get(hello)
        .push(user_api::routes());
        // .push( admin_routes)
    //添加swagger-ui
    let doc=OpenApi::new("app_server_api", "1.0.0")
        .add_security_scheme("bearer", SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer).bearer_format("JWT")))
        .merge_router(&router);
    let router=router.unshift(doc.into_router("/api-doc/openapi.json"))
    .unshift(SwaggerUi::new("/api-doc/openapi.json").into_router("/swagger-ui"));
    let service=Service::new(router).hoop(cors).hoop(Logger::new());
    service
}
