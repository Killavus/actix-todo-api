use crate::error::WebError;
use actix_web::{dev, web, Error, FromRequest, HttpRequest};
use futures::future::LocalBoxFuture;
use futures::prelude::*;

use serde::de::DeserializeOwned;
use validator::Validate;

pub struct ValidatedJson<T>(T);

impl<T> ValidatedJson<T> {
  pub fn into_inner(self) -> T {
    self.0
  }
}

impl<T: Validate + DeserializeOwned + 'static> FromRequest for ValidatedJson<T> {
  type Error = WebError;
  type Config = ();
  type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

  fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
    web::Json::<T>::from_request(req, payload)
      .map(|result: Result<web::Json<T>, Error>| match result {
        Ok(body) => {
          let inner = body.into_inner();

          match inner.validate() {
            Err(verr) => Err(WebError::ValidationError(verr)),
            Ok(_) => Ok(Self(inner)),
          }
        }
        Err(aerr) => Err(WebError::ActixError(aerr)),
      })
      .boxed_local()
  }
}
