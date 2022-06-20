use actix_web::error;
use actix_web::Error;
use actix_web::{FromRequest, HttpMessage};
use entity::session;
use futures::future::{ready, Ready};

use crate::constants;


pub struct Authenticated(entity::session::Model);

impl FromRequest for Authenticated {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req.extensions().get::<session::Model>().cloned();
        let result = match value {
            Some(v) => Ok(Authenticated(v)),
            None => Err(error::ErrorUnauthorized(constants::MESSAGE_INVALID_TOKEN)),
        };
        ready(result)
    }
}
impl std::ops::Deref for Authenticated {
    type Target = session::Model;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
