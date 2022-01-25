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
use crate::model::Model;
use crate::msg::Msg;
use crate::repository::RepositorySet;
use crate::usecase::{
    check_access_token, check_authcode, check_token_pair, create_authcode, create_token_pair,
};
use crate::utils::r#async::AsyncTryFrom;

#[derive(Component)]
pub struct Resolver {
    #[injected]
    repository: Injected<RepositorySet>,

    #[injected]
    command: Injected<CommandSet>,

    #[injected]
    config: Injected<Config>,
}

impl Resolver {
    async fn resolve(&self, msg: Msg) -> crate::Result<Model> {
        let repository = Arc::clone(&self.repository);
        let command = Arc::clone(&self.command);
        let config = Arc::clone(&self.config);

        let model = match msg {
            Msg::CreateAuthcode(payload) => create_authcode::execute(payload, repository, command)
                .await?
                .into(),

            Msg::CreateTokenPair(payload) => {
                let model = check_authcode::execute(payload.clone(), repository).await?;

                create_token_pair::execute(model.into(), command, config.secret_key())
                    .await?
                    .into()
            }

            Msg::RefreshTokenPair(payload) => {
                let user_id =
                    check_token_pair::execute(payload, command.clone(), config.secret_key())
                        .await?;

                create_token_pair::execute(user_id.into(), command, config.secret_key())
                    .await?
                    .into()
            }

            Msg::CheckAccessToken(payload) => {
                check_access_token::execute(payload, command, config.secret_key())
                    .await?
                    .into()
            }
        };

        Ok(model)
    }
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

async fn handler(request: Request<Body>, resolver: Arc<Resolver>) -> crate::Result<Response<Body>> {
    let msg = Msg::async_try_from(request).await?;

    let model = resolver.resolve(msg).await?;

    let response = model.into();

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

#[async_trait::async_trait]
impl ComponentLifecycle for HttpServer {
    async fn start(&mut self) {
        /* let (tx, rx) = mpsc::channel(8);

        self.tx.replace(tx);
        self.rx.replace(rx); */

        let resolver = Arc::clone(&self.resolver);

        let port = self.config.port();

        tokio::spawn(async move {
            let addr = SocketAddr::from(([0, 0, 0, 0], port));

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
            }
        });
    }

    async fn stop(&mut self) {}
}

/* pub async fn app(request: Request<Body>) -> crate::Result<Response<Body>> {
    /* let app = tower::service_fn(|request: Request<Body>| async move {
        let msg = Msg::from(&request);

        let model = resolve(msg).await;

        let response = present(model).await;

        response
    }); */

    let msg = Msg::try_from(request).await?;

    let model = resolve(msg).await?;

    let response = model.present()?;

    Ok(response)
} */
