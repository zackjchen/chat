use jwt_simple::prelude::*;

use crate::{error::AppError, User};
const JWT_DURATION: u64 = 60 * 60 * 24 * 7;
const JWT_AUDIENCE: &str = "chat_web";
const JWT_ISSUER: &str = "chat_server";

pub(crate) struct EncodingKey(Ed25519KeyPair);
pub(crate) struct DecodingKey(Ed25519PublicKey);

impl EncodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        Ok(Self(Ed25519KeyPair::from_pem(pem)?))
    }

    pub fn sign(&self, user: impl Into<User>) -> Result<String, AppError> {
        let claims = Claims::with_custom_claims(user.into(), Duration::from_secs(JWT_DURATION));
        let claims = claims.with_issuer("chat_server").with_audience("chat_web");
        Ok(self.0.sign(claims)?)
    }
}

impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        Ok(Self(Ed25519PublicKey::from_pem(pem)?))
    }
    #[allow(unused)]
    pub fn verify(&self, token: &str) -> Result<User, AppError> {
        let mut options = VerificationOptions {
            allowed_issuers: Some(HashSet::from_strings(&[JWT_ISSUER])),
            allowed_audiences: Some(HashSet::from_strings(&[JWT_AUDIENCE])),
            ..Default::default()
        };
        let claims = self.0.verify_token(token, Some(options))?;
        Ok(claims.custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_jwt() -> anyhow::Result<()> {
        let encoding_key = include_str!("../../fixtures/encoding_key.pem");
        let decoding_key = include_str!("../../fixtures/decoding_key.pem");
        let ek = EncodingKey::load(encoding_key)?;
        let dk = DecodingKey::load(decoding_key)?;
        let user_init = User {
            id: 1,
            ws_id: 0,
            fullname: "test".to_string(),
            email: "zackjchen@hkjc.org.hk".into(),
            password_hash: None,
            created_at: chrono::Utc::now(),
        };
        let token = ek.sign(user_init.clone())?;
        let user = dk.verify(&token)?;
        // assert_eq!(token, "");

        assert_eq!(user_init, user);
        Ok(())
    }
}
