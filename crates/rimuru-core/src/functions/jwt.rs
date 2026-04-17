use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("malformed JWT")]
    Malformed,
    #[error("unsupported algorithm: {0}")]
    UnsupportedAlg(String),
    #[error("invalid signature")]
    BadSignature,
    #[error("token expired")]
    Expired,
    #[error("missing exp claim")]
    MissingExp,
    #[error("encoding: {0}")]
    Encoding(String),
}

/// Minimum secret length in bytes for HS256. RFC 2104 recommends the HMAC key
/// be at least as long as the hash output (SHA-256 = 32 bytes).
pub const MIN_HS256_SECRET_BYTES: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

impl Claims {
    pub fn user(&self) -> Option<String> {
        self.user_id.clone().or_else(|| self.sub.clone())
    }
}

pub fn verify_hs256(token: &str, secret: &[u8]) -> Result<Claims, JwtError> {
    let mut parts = token.split('.');
    let header_b64 = parts.next().ok_or(JwtError::Malformed)?;
    let payload_b64 = parts.next().ok_or(JwtError::Malformed)?;
    let sig_b64 = parts.next().ok_or(JwtError::Malformed)?;
    if parts.next().is_some() {
        return Err(JwtError::Malformed);
    }

    let header_bytes = URL_SAFE_NO_PAD
        .decode(header_b64)
        .map_err(|e| JwtError::Encoding(e.to_string()))?;
    let header: Value =
        serde_json::from_slice(&header_bytes).map_err(|e| JwtError::Encoding(e.to_string()))?;
    let alg = header
        .get("alg")
        .and_then(|v| v.as_str())
        .ok_or(JwtError::Malformed)?;
    if alg != "HS256" {
        return Err(JwtError::UnsupportedAlg(alg.to_string()));
    }

    let signing_input = format!("{}.{}", header_b64, payload_b64);
    let expected_sig = URL_SAFE_NO_PAD
        .decode(sig_b64)
        .map_err(|e| JwtError::Encoding(e.to_string()))?;

    let mut mac = HmacSha256::new_from_slice(secret).map_err(|_| JwtError::BadSignature)?;
    mac.update(signing_input.as_bytes());
    mac.verify_slice(&expected_sig)
        .map_err(|_| JwtError::BadSignature)?;

    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|e| JwtError::Encoding(e.to_string()))?;
    let claims: Claims =
        serde_json::from_slice(&payload_bytes).map_err(|e| JwtError::Encoding(e.to_string()))?;

    let exp = claims.exp.ok_or(JwtError::MissingExp)?;
    let now = chrono::Utc::now().timestamp();
    if now >= exp {
        return Err(JwtError::Expired);
    }

    Ok(claims)
}

pub fn encode_hs256(claims: &Claims, secret: &[u8]) -> Result<String, JwtError> {
    let header = serde_json::json!({"alg": "HS256", "typ": "JWT"});
    let header_b64 = URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&header).map_err(|e| JwtError::Encoding(e.to_string()))?);
    let payload_b64 = URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(claims).map_err(|e| JwtError::Encoding(e.to_string()))?);
    let signing_input = format!("{}.{}", header_b64, payload_b64);
    let mut mac = HmacSha256::new_from_slice(secret).map_err(|_| JwtError::BadSignature)?;
    mac.update(signing_input.as_bytes());
    let sig = mac.finalize().into_bytes();
    let sig_b64 = URL_SAFE_NO_PAD.encode(sig);
    Ok(format!("{}.{}", signing_input, sig_b64))
}

pub fn extract_bearer(headers: &Value) -> Option<String> {
    let obj = headers.as_object()?;
    for (k, v) in obj {
        if k.eq_ignore_ascii_case("authorization") {
            let s = v.as_str()?;
            let trimmed = s.trim();
            if let Some(rest) = trimmed.strip_prefix("Bearer ") {
                return Some(rest.trim().to_string());
            }
            if let Some(rest) = trimmed.strip_prefix("bearer ") {
                return Some(rest.trim().to_string());
            }
        }
    }
    None
}

pub fn allow_without_jwt() -> bool {
    std::env::var("RIMURU_ALLOW_TEAM_WITHOUT_JWT")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

pub fn jwt_secret() -> Option<Vec<u8>> {
    let raw = std::env::var("RIMURU_JWT_SECRET").ok()?;
    if raw.is_empty() {
        tracing::warn!("RIMURU_JWT_SECRET is set but empty; ignoring");
        return None;
    }
    if raw.len() < MIN_HS256_SECRET_BYTES {
        tracing::warn!(
            "RIMURU_JWT_SECRET is shorter than {} bytes; ignoring for HS256 safety",
            MIN_HS256_SECRET_BYTES
        );
        return None;
    }
    Some(raw.into_bytes())
}

/// Enforce team-scoped auth on an incoming HTTP trigger input.
/// Returns the verified claims (or `None` when local-dev bypass is enabled).
pub fn authorize(input: &Value) -> Result<Option<Claims>, iii_sdk::IIIError> {
    let headers = input.get("headers").cloned().unwrap_or(Value::Null);
    let token = extract_bearer(&headers);

    match (token, jwt_secret(), allow_without_jwt()) {
        (Some(tok), Some(secret), _) => verify_hs256(&tok, &secret)
            .map(Some)
            .map_err(|e| iii_sdk::IIIError::Handler(format!("unauthorized: {}", e))),
        (None, _, true) => Ok(None),
        (Some(_), None, true) => {
            tracing::warn!(
                "RIMURU_ALLOW_TEAM_WITHOUT_JWT=1 bypassing presented bearer token (no RIMURU_JWT_SECRET configured); accepting as unauthenticated"
            );
            Ok(None)
        }
        _ => Err(iii_sdk::IIIError::Handler(
            "unauthorized: missing or invalid bearer token".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_missing_exp() {
        let secret = b"very-long-secret-key-of-at-least-32b";
        let claims = Claims {
            sub: Some("alice".into()),
            user_id: None,
            team_id: None,
            exp: None,
            extra: Default::default(),
        };
        let token = encode_hs256(&claims, secret).unwrap();
        let result = verify_hs256(&token, secret);
        assert!(matches!(result, Err(JwtError::MissingExp)));
    }

    #[test]
    fn claims_omit_none_fields_on_serialize() {
        let claims = Claims {
            sub: None,
            user_id: Some("alice".into()),
            team_id: None,
            exp: Some(123),
            extra: Default::default(),
        };
        let s = serde_json::to_string(&claims).unwrap();
        assert!(!s.contains("\"sub\""));
        assert!(!s.contains("\"team_id\""));
        assert!(s.contains("\"user_id\""));
        assert!(s.contains("\"exp\""));
    }

    #[test]
    fn jwt_secret_rejects_empty_and_short() {
        // Use a unique var name per test to avoid clobbering real env.
        // We serialize via a mutex because std::env is process-global.
        use std::sync::Mutex;
        static LOCK: Mutex<()> = Mutex::new(());
        let _g = LOCK.lock().unwrap();

        // SAFETY: set/remove env in a single-threaded critical section.
        unsafe {
            std::env::remove_var("RIMURU_JWT_SECRET");
            assert!(jwt_secret().is_none());

            std::env::set_var("RIMURU_JWT_SECRET", "");
            assert!(jwt_secret().is_none());

            std::env::set_var("RIMURU_JWT_SECRET", "too-short");
            assert!(jwt_secret().is_none());

            std::env::set_var(
                "RIMURU_JWT_SECRET",
                "this-is-a-sufficiently-long-hs256-key-xx",
            );
            assert!(jwt_secret().is_some());

            std::env::remove_var("RIMURU_JWT_SECRET");
        }
    }
}
