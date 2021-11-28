mod graphviz;

use std::net::ToSocketAddrs;

use futures::stream::StreamExt;
use signal_hook::consts::signal::{SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use signal_hook_tokio::Signals;
use structopt::StructOpt;
use warp::Filter;

use crate::graphviz::{
    warp::{accept_format_or, dot, handle_rejection},
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
    let mut signals = Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT]).unwrap();
    let signal = async move {
        signals.next().await;
    };

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
        .and(warp::fs::file("src/template.html"));
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
    let status = warp::path!("status").and(warp::get()).map(|| {
        if graphviz::is_rate_limited() {
            "LIMITED"
        } else {
            "OK"
        }
    });
    let app = api.or(page).or(icon).or(status).recover(handle_rejection);

    let socket_addr = (opts.host, opts.port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    let (_, server) = warp::serve(app).bind_with_graceful_shutdown(socket_addr, signal);
    server.await;
}
