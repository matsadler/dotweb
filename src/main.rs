mod graphviz;

use std::net::ToSocketAddrs;

use structopt::StructOpt;
use warp::Filter;

use crate::graphviz::{
    warp::{accept_format_or, dot},
    Format,
};

/// Web interface for dot
#[derive(StructOpt, Debug)]
pub struct Opts {
    /// Host to listen on
    #[structopt(short = "H", long, default_value = "127.0.0.1", value_name = "HOST")]
    pub host: String,
    /// Port to listen on
    #[structopt(short = "P", long, default_value = "8080", value_name = "PORT")]
    pub port: u16,
}

#[tokio::main]
async fn main() {
    let opts = Opts::from_args();

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

    let socket_addr = (opts.host, opts.port)
        .to_socket_addrs().unwrap()
        .next()
        .unwrap();
    warp::serve(app).run(socket_addr).await;
}
