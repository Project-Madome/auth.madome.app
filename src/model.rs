use crate::{
    into_model,
    usecase::{check_access_token, create_authcode, create_token_pair},
};

into_model![
    (CreateAuthcode, create_authcode::Model),
    (CreateTokenPair, create_token_pair::Model),
    (CheckAccessToken, check_access_token::Model),
];

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
