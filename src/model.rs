use hyper::{header, http::response::Builder as ResponseBuilder, Body, Response, StatusCode};
use madome_sdk::api::cookie::{MADOME_ACCESS_TOKEN, MADOME_REFRESH_TOKEN};
use util::http::{SetCookie, SetCookieOptions, SetHeaders};

use crate::{
    into_model,
    usecase::{
        check_access_token, check_and_refresh_token_pair, create_authcode, create_token_pair,
        delete_token_pair,
    },
};

#[cfg_attr(test, derive(Default))]
#[derive(Debug)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

into_model![
    (TokenPair, TokenPair),
    (CreateAuthcode, create_authcode::Model),
    (CheckAccessToken, check_access_token::Model),
    (
        CheckAndRefreshTokenPair,
        check_and_refresh_token_pair::Model
    ),
    (CreateTokenPair, create_token_pair::Model),
    (DeleteTokenPair, delete_token_pair::Model),
];

pub trait Presenter: Sized {
    fn to_http(self, _response: ResponseBuilder) -> Response<Body> {
        unimplemented!()
    }
}

impl Presenter for create_authcode::Model {
    fn to_http(self, response: ResponseBuilder) -> Response<Body> {
        response
            .status(StatusCode::CREATED)
            .body(Body::empty())
            .unwrap()
    }
}

impl From<TokenPair> for SetCookie {
    fn from(model: TokenPair) -> Self {
        let set_cookie_options = SetCookieOptions::new()
            .domain("madome.app")
            .path("/")
            .http_only(true)
            .secure(true);

        SetCookie::new()
            .set(
                MADOME_ACCESS_TOKEN,
                model.access_token,
                set_cookie_options.clone().max_age(3600 * 24 * 7),
            )
            .set(
                MADOME_REFRESH_TOKEN,
                model.refresh_token,
                set_cookie_options.max_age(3600 * 24 * 7),
            )
    }
}

impl Presenter for TokenPair {
    fn to_http(self, response: ResponseBuilder) -> Response<Body> {
        let set_cookie = SetCookie::from(self);

        /* log::debug!(
            "set-cookie = {:?}",
            set_cookie
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_str().unwrap().to_string()))
                .collect::<Vec<_>>()
        ); */

        response
            .status(StatusCode::CREATED)
            .headers(set_cookie.iter())
            .body(Body::empty())
            .unwrap()
    }
}

impl Presenter for check_access_token::Model {
    fn to_http(self, response: ResponseBuilder) -> Response<Body> {
        let serialized = serde_json::to_vec(&self).expect("json serialize");

        response
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serialized.into())
            .unwrap()
    }
}

impl Presenter for check_and_refresh_token_pair::Model {
    fn to_http(self, mut response: ResponseBuilder) -> Response<Body> {
        let serialized = serde_json::to_vec(&self).expect("json serialize");

        if let (Some(access_token), Some(refresh_token)) = (self.access_token, self.refresh_token) {
            let token_pair = TokenPair {
                access_token,
                refresh_token,
            };
            response = response.headers(SetCookie::from(token_pair).iter());
        }

        response
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serialized.into())
            .unwrap()
    }
}

impl Presenter for create_token_pair::Model {
    fn to_http(self, response: ResponseBuilder) -> Response<Body> {
        let token_pair = TokenPair {
            access_token: self.access_token,
            refresh_token: self.refresh_token,
        };

        token_pair.to_http(response)
    }
}

impl Presenter for delete_token_pair::Model {
    fn to_http(self, response: ResponseBuilder) -> Response<Body> {
        let max_age_0 = SetCookieOptions::new()
            .domain("madome.app")
            .path("/")
            .http_only(true)
            .secure(true)
            .max_age(0);

        let set_cookie = SetCookie::new()
            .set(MADOME_ACCESS_TOKEN, "", max_age_0.clone())
            .set(MADOME_REFRESH_TOKEN, "", max_age_0);

        let response = response.headers(set_cookie.iter());

        response
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .unwrap()
    }
}

#[macro_export]
macro_rules! into_model {
    ($(($member:ident, $from:ty)),*,) => {
        pub enum Model {
            $(
                $member($from),
            )*
        }

        $(
            impl From<$from> for Model {
                fn from(from: $from) -> Model {
                    Model::$member(from)
                }
            }
        )*


        impl Presenter for Model {
            fn to_http(self, response: ResponseBuilder) -> Response<Body> {
                use Model::*;

                match self {
                    $(
                        $member(model) => model.to_http(response),
                    )*
                }
            }
        }
    };
}
