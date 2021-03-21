use warp::{
    http::{
        header::{HeaderValue, CONTENT_TYPE},
        Response,
    },
    hyper::Body,
    reject::{Reject, Rejection},
    Filter, Reply,
};

use super::{Error, Format, Gif, Jpg, OutputFormat, Pdf, Png, Postscript, Svg};

impl Reject for Error {}

macro_rules! reply {
    ($T:ty, $content_type:literal) => {
        impl Reply for $T {
            fn into_response(self) -> Response<Body> {
                Response::builder()
                    .header(CONTENT_TYPE, HeaderValue::from_static($content_type))
                    .body(Body::from(self.0))
                    .unwrap()
            }
        }
    };
}

reply!(Gif, "image/gif");
reply!(Jpg, "image/jpeg");
reply!(Pdf, "application/pdf");
reply!(Png, "image/png");
reply!(Postscript, "application/postscript");
reply!(Svg, "image/svg+xml");

impl Reply for OutputFormat {
    fn into_response(self) -> Response<Body> {
        match self {
            OutputFormat::Gif(gif) => gif.into_response(),
            OutputFormat::Jpg(jpg) => jpg.into_response(),
            OutputFormat::Pdf(pdf) => pdf.into_response(),
            OutputFormat::Png(png) => png.into_response(),
            OutputFormat::Postscript(ps) => ps.into_response(),
            OutputFormat::Svg(svg) => svg.into_response(),
        }
    }
}

pub fn accept_format_or(
    default: Format,
) -> impl Filter<Extract = (Format,), Error = Rejection> + Copy {
    warp::any().and(
        warp::header::optional("Accept").map(move |t: Option<String>| {
            t.as_deref()
                .and_then(Format::from_content_type)
                .unwrap_or(default)
        }),
    )
}

pub async fn dot(bytes: bytes::Bytes, format: Format) -> Result<OutputFormat, Rejection> {
    super::dot_with_format(&bytes, format)
        .await
        .map_err(warp::reject::custom)
}
