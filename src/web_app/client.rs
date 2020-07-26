use crate::database::Pool;
use crate::error::WebError;
use actix_web::{dev, web, FromRequest, HttpRequest};
use futures::future::{ready, LocalBoxFuture};
use futures::prelude::*;
use log::warn;
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct Client {
    id: i64,
    display_name: String,
}

impl Client {
    pub fn id(&self) -> i64 {
        self.id
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
        let pool = web::Data::<Pool>::from_request(req, payload)
            .into_inner()
            .map_err(|err| {
                warn!("Failed to obtain database pool: {:?}", err);
                WebError::Unauthorized
            });

        let token = req
            .headers()
            .get("Authorization")
            .ok_or(WebError::Unauthorized)
            .and_then(|header| header.to_str().map_err(|_| WebError::Unauthorized))
            .map(|header| header.to_lowercase())
            .and_then(|header| {
                if header.starts_with("token") {
                    Ok(header
                        .trim_start_matches("token")
                        .trim_start()
                        .trim_end()
                        .to_string())
                } else {
                    Err(WebError::Unauthorized)
                }
            });

        if token.is_err() || pool.is_err() {
            return ready(Err(WebError::Unauthorized)).boxed_local();
        } else {
            // SAFETY: Safe because we've checked for error case in conditional.
            let pool = pool.unwrap();
            let token = token.unwrap();
            let fut = async move {
                Self::authorize(&token, &pool)
                    .await
                    .and_then(|client| client.ok_or(WebError::Unauthorized))
            };

            fut.boxed_local()
        }
    }
}
