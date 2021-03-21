mod graphviz;

use warp::Filter;

use crate::graphviz::{
    warp::{accept_format_or, dot},
    Format,
};

#[tokio::main]
async fn main() {
    let api = warp::path::end()
        .and(warp::post())
        .and(
            warp::header::exact("Content-Type", "text/vnd.graphviz")
                .or(warp::header::exact("Content-Type", "text/plain"))
                .unify(),
        )
        .and(warp::body::content_length_limit(1024 * 64))
        .and(warp::body::bytes())
        .and(accept_format_or(Format::Png))
        .and_then(dot);
    #[cfg(debug_assertions)]
    let page = warp::path::end()
        .and(warp::get())
        .and(warp::fs::file("template.html"));
    #[cfg(not(debug_assertions))]
    let page = warp::path::end()
        .and(warp::get())
        .map(|| warp::reply::html(include_str!("template.html")));
    let icon = warp::path!("favicon.ico").and(warp::get()).map(|| {
        warp::reply::with_header(
            &include_bytes!("favicon.ico")[..],
            "Content-Type",
            "image/vnd.microsoft.icon",
        )
    });
    let app = api.or(page).or(icon);

    warp::serve(app).run(([127, 0, 0, 1], 8080)).await;
}
