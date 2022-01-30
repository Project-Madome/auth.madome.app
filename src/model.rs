use hyper::{Body, Response, StatusCode};
use util::http::{SetCookie, SetCookieOptions, SetHeaders};

use crate::{
    constant::http::cookie::{MADOME_ACCESS_TOKEN, MADOME_REFRESH_TOKEN},
    into_model,
    usecase::{check_access_token, create_authcode, create_token_pair},
};

into_model![
    (CreateAuthcode, create_authcode::Model),
    (CreateTokenPair, create_token_pair::Model),
    (CheckAccessToken, check_access_token::Model),
];

impl From<create_authcode::Model> for Response<Body> {
    fn from(_: create_authcode::Model) -> Self {
        Response::builder()
            .status(StatusCode::CREATED)
            .body(Body::empty())
            .unwrap()
    }
}

impl From<create_token_pair::Model> for Response<Body> {
    fn from(
        create_token_pair::Model {
            access_token,
            refresh_token,
        }: create_token_pair::Model,
    ) -> Self {
        let set_cookie_options = SetCookieOptions::new().domain("madome.app").http_only(true);
        let set_cookie = SetCookie::new()
            .set(
                MADOME_ACCESS_TOKEN,
                access_token,
                set_cookie_options.clone().max_age(3600 * 4),
            )
            .set(
                MADOME_REFRESH_TOKEN,
                refresh_token,
                set_cookie_options.max_age(3600 * 24 * 7),
            );

        Response::builder()
            .status(StatusCode::CREATED)
            .headers(set_cookie.iter())
            .body(Body::empty())
            // .header(header::CONTENT_TYPE, header::APPLICATION_JSON)
            // .body(Body::from(serialized))
            .unwrap()
    }
}

impl From<check_access_token::Model> for Response<Body> {
    fn from(model: check_access_token::Model) -> Self {
        let serialized = serde_json::to_string(&model).expect("json serialize");

        Response::builder()
            .status(StatusCode::OK)
            .body(serialized.into())
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


        impl From<Model> for hyper::Response<hyper::Body> {
            fn from(model: Model) -> Self {
                use Model::*;

                match model {
                    $(
                        $member(model) => model.into(),
                    )*
                }
            }
        }

    };
}
