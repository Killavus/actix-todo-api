use crate::database::Pool;
use crate::error::WebError;
use actix_web::{dev, web, FromRequest, HttpRequest};
use futures::future::{ready, LocalBoxFuture};
use futures::prelude::*;
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Client {
    id: i64,
    display_name: String,
}

impl Client {
    fn new(id: i64, display_name: String) -> Self {
        Self { id, display_name }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.display_name
    }

    pub async fn authorize(token: &str, pool: &Pool) -> Result<Option<Self>, WebError> {
        let client_id: Option<(i64,)> = sqlx::query_as("SELECT client_id FROM client_api_keys WHERE key = ? AND valid_to > strftime('%s','now')").bind(token).fetch_optional(&*pool).await?;

        if let Some((id,)) = client_id {
            sqlx::query_as!(Client, "SELECT * FROM clients WHERE id = ?", id)
                .fetch_optional(&*pool)
                .await
                .map_err(|err| err.into())
        } else {
            Ok(None)
        }
    }
}

impl FromRequest for Client {
    type Error = WebError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
        if let Some(auth) = req.headers().get("Authorization") {
            match auth.to_str() {
                Ok(auth_str) => {
                    let mut token = auth_str.split_whitespace().skip(1);

                    if let Some(token) = token.next() {
                        let token = token.to_lowercase();
                        match req
                            .app_data::<Pool>()
                            .ok_or(WebError::Unauthorized)
                            .map(|pool| Client::authorize(&token, &pool))
                        {
                            Ok(fut) => fut
                                .map(|client| client.unwrap_or(None).ok_or(WebError::Unauthorized))
                                .boxed_local(),
                            Err(err) => ready(Err(err)).boxed_local(),
                        }
                    } else {
                        ready(Err(WebError::Unauthorized)).boxed_local()
                    }
                }
                Err(_) => ready(Err(WebError::Unauthorized)).boxed_local(),
            }
        } else {
            ready(Err(WebError::Unauthorized)).boxed_local()
        }
    }
}
