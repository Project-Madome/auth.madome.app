use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::{convert::Infallible, net::SocketAddr};

use hyper::{
    body::Body,
    http::{Request, Response},
    service::{make_service_fn, service_fn},
};
use inspect::{Inspect, InspectOk};
use sai::{Component, ComponentLifecycle, Injected};

use crate::command::CommandSet;
use crate::config::Config;
use crate::model::{Model, Presenter};
use crate::msg::Msg;
use crate::repository::RepositorySet;
use crate::usecase::{
    check_access_token, check_and_refresh_token_pair, check_authcode, check_token_pair,
    create_authcode, create_token_pair,
};

#[cfg_attr(test, derive(Default))]
#[derive(Component)]
pub struct Resolver {
    #[injected]
    repository: Injected<RepositorySet>,

    #[injected]
    command: Injected<CommandSet>,
    /* #[injected]
    config: Injected<Config>, */
}

impl Resolver {
    async fn resolve(&self, msg: Msg) -> crate::Result<Model> {
        let repository = Arc::clone(&self.repository);
        let command = Arc::clone(&self.command);
        // let config = Arc::clone(&self.config);

        let model = match msg {
            Msg::CreateAuthcode(payload) => create_authcode::execute(payload, repository, command)
                .await?
                .into(),

            Msg::CreateTokenPair(payload) => {
                let model = check_authcode::execute(payload.clone(), repository.clone()).await?;

                create_token_pair::execute(model.into(), repository, command)
                    .await?
                    .into()
            }

            Msg::RefreshTokenPair(payload) => {
                let user_id =
                    check_token_pair::execute(payload, repository.clone(), command.clone()).await?;

                create_token_pair::execute(user_id.into(), repository, command)
                    .await?
                    .into()
            }

            Msg::CheckAccessToken(payload) => {
                check_access_token::execute(payload, repository, command)
                    .await?
                    .into()
            }

            Msg::CheckAndRefreshTokenPair(payload) => {
                check_and_refresh_token_pair::execute(payload, repository, command)
                    .await?
                    .into()
            }
        };

        Ok(model)
    }
}

async fn handler(request: Request<Body>, resolver: Arc<Resolver>) -> crate::Result<Response<Body>> {
    let response = Response::builder();

    let (msg, response) = Msg::from_http(request, response).await?;

    let model = resolver.resolve(msg).await?;

    let response = model.to_http(response);

    Ok(response)
}

async fn service(
    request: Request<Body>,
    resolver: Arc<Resolver>,
) -> Result<Response<Body>, Infallible> {
    let req_method = request.method().to_owned();
    let req_uri = request.uri().to_string();

    log::info!("HTTP Request {} {}", req_method, req_uri);

    let start = SystemTime::now();

    let response = handler(request, resolver).await;

    let end = start
        .elapsed()
        .as_ref()
        .map(Duration::as_millis)
        .unwrap_or(0);

    match response {
        Ok(response) => Ok(response),
        Err(err) => Ok(err.inspect(|e| log::error!("{}", e)).into()),
    }
    .inspect_ok(|res| {
        log::info!(
            "HTTP Response {} {} {} {}ms",
            req_method,
            req_uri,
            res.status(),
            end
        )
    })

    // log::error!("{}", err);
}

#[derive(Component)]
#[lifecycle]
pub struct HttpServer {
    #[injected]
    resolver: Injected<Resolver>,
    /* tx: Option<mpsc::Sender<()>>,
    rx: Option<mpsc::Receiver<()>>, */
    #[injected]
    config: Injected<Config>,
}

#[async_trait::async_trait]
impl ComponentLifecycle for HttpServer {
    async fn start(&mut self) {
        /* let (tx, rx) = mpsc::channel(8);

        self.tx.replace(tx);
        self.rx.replace(rx); */

        let resolver = Arc::clone(&self.resolver);

        let port = self.config.port();
        let addr = SocketAddr::from(([0, 0, 0, 0], port));

        let t = tokio::spawn(async move {
            let svc = |resolver: Arc<Resolver>| async move {
                Ok::<_, Infallible>(service_fn(move |request| {
                    service(request, Arc::clone(&resolver))
                }))
            };

            let server = hyper::Server::bind(&addr)
                .serve(make_service_fn(move |_| svc(Arc::clone(&resolver))));

            log::info!("started http server: 0.0.0.0:{}", port);

            if let Err(err) = server.await {
                panic!("{:?}", err);
                // oneshot 채널 열어서 스탑 메세지 보내서 서버 프로세스를 죽여야됨
            }
            /* // Server Mock
            let svc = |resolver: Arc<Resolver>| async move {
                Ok::<_, Infallible>(service_fn(move |request| {
                    service(request, Arc::clone(&resolver))
                }))
            };

            let server = hyper::Server::bind(&addr)
                .serve(make_service_fn(move |_| svc(Arc::clone(&resolver))));

            log::info!("started http server: 0.0.0.0:{}", port);

            if let Err(err) = server.await {
                panic!("{:?}", err);
            } */
        });

        t.await.unwrap();
    }

    async fn stop(&mut self) {}
}

/* #[cfg(test)]
mod tests {
    use std::{convert::Infallible, fs, sync::Arc};

    use hyper::{
        service::{make_service_fn, service_fn},
        Client,
    };
    use hyperlocal::{UnixClientExt, UnixServerExt};
    use sai::{Component, ComponentLifecycle, Injected};
    use util::test_registry;
    use uuid::Uuid;

    use crate::{
        command::{self, CommandSet},
        entity::token::Token,
        json::user::UserInfo,
        repository::RepositorySet,
    };

    use super::{service, Resolver};

    const SOCKET_PATH: &str = "aaaaa";

    struct MockHttpServer {
        pub resolver: Injected<Resolver>,
    }

    impl MockHttpServer {
        async fn start(&self, socket_path: &str) {
            let resolver = Arc::clone(&self.resolver);

            tokio::spawn(async move {
                let svc = |resolver: Arc<Resolver>| async move {
                    Ok::<_, Infallible>(service_fn(move |request| {
                        service(request, Arc::clone(&resolver))
                    }))
                };

                fs::create_dir_all("./.temp").expect("create dir");

                let server = hyper::Server::bind_unix(format!("./.temp/{}", socket_path))
                    .unwrap()
                    .serve(make_service_fn(move |_| svc(Arc::clone(&resolver))));

                if let Err(err) = server.await {
                    panic!("{:?}", err);
                }
            });
        }
    }

    #[tokio::test]
    async fn t() {
        use crate::repository::r#trait::SecretKeyRepository;

        test_registry!(
        [(Injected) -> repository: RepositorySet, command: CommandSet] ->
        [auth_socket_path: &str, user_id: String, secret_key: String, token: Token] ->
        {
            user_id = Uuid::new_v4().to_string();
            secret_key = "S3Cr#tK3y".to_string();
            token = Token::new(user_id.clone());

            repository
                .secret_key()
                .add(&token.id, &secret_key)
                .await
                .unwrap();

            let get_user_info = command::tests::GetUserInfo::from(UserInfo {
                id: user_id.clone(),
                email: "".to_string(),
                role: 0,
            });

            command.set_get_user_info(get_user_info);
        },
        {
            let resolver = Resolver {
                repository,
                command,
            };
            let mock_http_server = MockHttpServer {
                resolver: Injected::new(resolver)
            };

            tokio::spawn(async move {
                mock_http_server.start().await;
            });

            /* test code */

            let client = Client::unix();

            client.get()
        });
    }
} */
