use hyper::{
    header::{HeaderName, HeaderValue},
    http::response,
};

pub trait SetHeaders {
    fn headers(self, headers: impl Iterator<Item = (HeaderName, HeaderValue)>) -> Self;
}

impl SetHeaders for response::Builder {
    fn headers(self, headers: impl Iterator<Item = (HeaderName, HeaderValue)>) -> Self {
        headers.fold(self, |res, (key, value)| res.header(key, value))
    }
}
