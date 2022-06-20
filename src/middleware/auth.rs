use std::{
    future::{ready, Ready},
    rc::Rc,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{HeaderName, HeaderValue},
    web, Error, HttpMessage,
};

use crate::{constants, utils::token_utils};
use entity::session;
use futures::{future::LocalBoxFuture, FutureExt};
use log::info;
use sea_orm::DatabaseConnection;

impl<S, B> Transform<S, ServiceRequest> for AuthenticateMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthNMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthNMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthNMiddleware<S> {
    service: Rc<S>,
}

pub struct AuthenticateMiddlewareFactory {}

impl<S, B> Service<ServiceRequest> for AuthNMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();
        async move {
            let  headers = req.headers_mut();
            headers.append(
                HeaderName::from_static("content-length"),
                HeaderValue::from_static("true"),
            );

            if let Some(db) = req.app_data::<web::Data<DatabaseConnection>>() {
                if let Some(authen_header) = req.headers().get(constants::AUTHORIZATION) {
                    info!("Parsing authorization header...");
                    if let Ok(authen_str) = authen_header.to_str() {
                        if authen_str.starts_with("bearer") || authen_str.starts_with("Bearer") {
                            let token = authen_str[6..authen_str.len()].trim();
                            info!("Parsing token: {}", token.to_string());
                            let token_data = token_utils::decode_token(token.to_string());
                            if let Err(token_err) = &token_data {
                                info!("{:?}", &token_err);
                            }
                            if let Ok(token_val) = &token_data {
                                info!("Token Parsed");

                                if let Ok(_) = token_utils::verify_token(&token_val, db).await {
                                    info!("User Authenticated, Adding context to post");
                                    req.extensions_mut()
                                        .insert::<session::Model>(token_val.claims.clone());
                                } else {
                                    info!("Token not valid");
                                }
                            }
                        }
                    }
                }
            }

            let res = srv.call(req).await?;
            Ok(res)
        }
        .boxed_local()
    }
}
