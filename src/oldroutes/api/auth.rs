use actix_web::{get, post, web, HttpResponse};



#[get("/login")]
async fn user_login(tmpl: web::Data<tera::Tera>) -> HttpResponse {
    let s = tmpl.render("login.html", &tera::Context::new()).unwrap();
    HttpResponse::Ok().content_type("text/html").body(s)
}

