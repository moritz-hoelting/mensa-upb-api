use std::{net::IpAddr, str::FromStr};

use actix_governor::{
    governor::{
        clock::{Clock, DefaultClock, QuantaInstant},
        middleware::NoOpMiddleware,
    },
    GovernorConfig, GovernorConfigBuilder, KeyExtractor, SimpleKeyExtractionError,
};
use actix_web::{
    dev::ServiceRequest,
    http::{header::ContentType, StatusCode},
    HttpResponse, HttpResponseBuilder,
};
use serde::{Deserialize, Serialize};

use crate::USE_X_FORWARDED_HOST;

pub fn get_governor(
    seconds_replenish: u64,
    burst_size: u32,
) -> GovernorConfig<UserToken, NoOpMiddleware<QuantaInstant>> {
    GovernorConfigBuilder::default()
        .seconds_per_request(seconds_replenish)
        .burst_size(burst_size)
        .key_extractor(UserToken)
        .finish()
        .unwrap()
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct UserToken;

impl KeyExtractor for UserToken {
    type Key = IpAddr;
    type KeyExtractionError = SimpleKeyExtractionError<&'static str>;

    fn name(&self) -> &'static str {
        "Bearer token"
    }

    fn extract(&self, req: &ServiceRequest) -> Result<Self::Key, Self::KeyExtractionError> {
        let mut ip = USE_X_FORWARDED_HOST
            .then(|| {
                req.headers()
                    .get("X-Forwarded-Host")
                    .and_then(|header| IpAddr::from_str(header.to_str().unwrap_or_default()).ok())
            })
            .flatten()
            .or_else(|| req.peer_addr().map(|socket| socket.ip()))
            .ok_or_else(|| {
                Self::KeyExtractionError::new(
                    r#"{ "code": 500, "msg": "Could not extract peer IP address from request"}"#,
                )
                .set_content_type(ContentType::json())
                .set_status_code(StatusCode::INTERNAL_SERVER_ERROR)
            })?;

        // customers often get their own /56 prefix, apply rate-limiting per prefix instead of per
        // address for IPv6
        if let IpAddr::V6(ipv6) = ip {
            let mut octets = ipv6.octets();
            octets[7..16].fill(0);
            ip = IpAddr::V6(octets.into());
        }
        Ok(ip)
    }

    fn exceed_rate_limit_response(
        &self,
        negative: &actix_governor::governor::NotUntil<QuantaInstant>,
        mut response: HttpResponseBuilder,
    ) -> HttpResponse {
        let wait_time = negative
            .wait_time_from(DefaultClock::default().now())
            .as_secs();
        response.content_type(ContentType::json())
            .body(
                format!(
                    r#"{{"code":429, "error": "TooManyRequests", "message": "Too many requests, try again after {wait_time} seconds", "after": {wait_time}}}"#
                )
            )
    }

    fn whitelisted_keys(&self) -> Vec<Self::Key> {
        Vec::new()
    }

    fn key_name(&self, _key: &Self::Key) -> Option<String> {
        None
    }
}
