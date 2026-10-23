use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware::Next,
    response::Response,
};
use redis::Script;
use redis::aio::ConnectionManager;
use std::net::{IpAddr, SocketAddr};

use crate::utils::types::AppState;

pub const MAX_REQUESTS: u64 = 15;
pub const WINDOW_SECONDS: u64 = 60;

const KEY_PREFIX: &str = "pt_backend:rate_limit:v1:";
const INCREMENT_SCRIPT: &str = r#"
local count = redis.call('INCR', KEYS[1])
if count == 1 then
    redis.call('EXPIRE', KEYS[1], ARGV[1])
end
return { count, redis.call('TTL', KEYS[1]) }
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateLimitDecision {
    pub allowed: bool,
    pub remaining: u64,
    pub retry_after_seconds: u64,
}

impl RateLimitDecision {
    pub fn from_count(count: u64, ttl_seconds: i64) -> Self {
        let retry_after_seconds = ttl_seconds.max(1) as u64;
        Self {
            allowed: count <= MAX_REQUESTS,
            remaining: MAX_REQUESTS.saturating_sub(count),
            retry_after_seconds,
        }
    }
}

/// Extract the client address set by Vercel's proxy.
///
/// Vercel supplies `x-real-ip` for the client address. `x-forwarded-for` is
/// used as a fallback; its left-most valid address is the original client.
pub fn client_ip(headers: &HeaderMap) -> Option<IpAddr> {
    headers
        .get("x-real-ip")
        .and_then(|value| value.to_str().ok())
        .and_then(parse_ip)
        .or_else(|| {
            headers
                .get("x-forwarded-for")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.split(',').find_map(parse_ip))
        })
}

fn parse_ip(value: &str) -> Option<IpAddr> {
    let value = value.trim();
    value
        .parse::<IpAddr>()
        .ok()
        .or_else(|| value.parse::<SocketAddr>().ok().map(|address| address.ip()))
        .or_else(|| {
            value
                .strip_prefix('[')
                .and_then(|v| v.strip_suffix(']'))?
                .parse()
                .ok()
        })
}

fn redis_key(ip: IpAddr) -> String {
    format!("{KEY_PREFIX}{ip}")
}

async fn increment(redis: &mut ConnectionManager, ip: IpAddr) -> redis::RedisResult<(u64, i64)> {
    Script::new(INCREMENT_SCRIPT)
        .key(redis_key(ip))
        .arg(WINDOW_SECONDS)
        .invoke_async(redis)
        .await
}

pub async fn rate_limit(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let Some(ip) = client_ip(request.headers()) else {
        return error_response(
            StatusCode::BAD_REQUEST,
            "A valid client IP address is required",
        );
    };

    let mut redis = state.redis.clone();
    let (count, ttl_seconds) = match increment(&mut redis, ip).await {
        Ok(result) => result,
        Err(_) => {
            // Failing closed prevents a Redis outage from silently bypassing
            // the limit and makes the dependency failure visible to callers.
            return error_response(StatusCode::SERVICE_UNAVAILABLE, "Rate limiter unavailable");
        }
    };

    let decision = RateLimitDecision::from_count(count, ttl_seconds);
    if !decision.allowed {
        let mut response = error_response(StatusCode::TOO_MANY_REQUESTS, "Too many requests");
        response.headers_mut().insert(
            header::RETRY_AFTER,
            HeaderValue::from(decision.retry_after_seconds),
        );
        response
            .headers_mut()
            .insert("x-ratelimit-limit", HeaderValue::from(MAX_REQUESTS));
        response
            .headers_mut()
            .insert("x-ratelimit-remaining", HeaderValue::from(0u64));
        return response;
    }

    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("x-ratelimit-limit", HeaderValue::from(MAX_REQUESTS));
    response.headers_mut().insert(
        "x-ratelimit-remaining",
        HeaderValue::from(decision.remaining),
    );
    response
}

fn error_response(status: StatusCode, message: &'static str) -> Response {
    Response::builder()
        .status(status)
        .body(Body::from(message))
        .expect("valid rate limiter response")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn extracts_the_leftmost_forwarded_ip() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.9, 10.0.0.2"),
        );
        assert_eq!(client_ip(&headers), Some("203.0.113.9".parse().unwrap()));
    }

    #[test]
    fn skips_invalid_forwarded_entries_and_uses_real_ip_fallback() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("not-an-ip"));
        headers.insert("x-real-ip", HeaderValue::from_static("198.51.100.4"));
        assert_eq!(client_ip(&headers), Some("198.51.100.4".parse().unwrap()));
    }

    #[test]
    fn prefers_vercels_real_ip_when_both_proxy_headers_are_present() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", HeaderValue::from_static("198.51.100.4"));
        headers.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.9"));
        assert_eq!(client_ip(&headers), Some("198.51.100.4".parse().unwrap()));
    }

    #[test]
    fn accepts_ipv6_and_socket_address_header_values() {
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", HeaderValue::from_static("[2001:db8::1]:443"));
        assert_eq!(client_ip(&headers), Some("2001:db8::1".parse().unwrap()));
    }

    #[test]
    fn rejects_requests_without_a_valid_ip() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("unknown"));
        assert_eq!(client_ip(&headers), None);
    }

    #[test]
    fn allows_exactly_24_requests_and_rejects_the_25th() {
        let allowed = RateLimitDecision::from_count(24, 41);
        assert!(allowed.allowed);
        assert_eq!(allowed.remaining, 0);

        let rejected = RateLimitDecision::from_count(25, 41);
        assert!(!rejected.allowed);
        assert_eq!(rejected.remaining, 0);
        assert_eq!(rejected.retry_after_seconds, 41);
    }

    #[test]
    fn retry_after_is_at_least_one_second_for_expired_or_missing_ttl() {
        assert_eq!(RateLimitDecision::from_count(25, 0).retry_after_seconds, 1);
        assert_eq!(RateLimitDecision::from_count(25, -1).retry_after_seconds, 1);
    }

    #[test]
    fn creates_a_separate_redis_bucket_for_each_ip() {
        assert_ne!(
            redis_key("203.0.113.9".parse().unwrap()),
            redis_key("203.0.113.10".parse().unwrap())
        );
    }
}
