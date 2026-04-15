use super::*;

const ADMIN_JWT_ISSUER: &str = "sdkwork-admin";
const ADMIN_JWT_AUDIENCE: &str = "sdkwork-admin-ui";
const ADMIN_JWT_TTL_SECS: u64 = 60 * 60 * 12;

pub(crate) const PORTAL_JWT_ISSUER: &str = "sdkwork-portal";
pub(crate) const PORTAL_JWT_AUDIENCE: &str = "sdkwork-public-portal";
pub(crate) const PORTAL_JWT_TTL_SECS: u64 = 60 * 60 * 12;

type HmacSha256 = Hmac<JwtSha256>;

static HMAC_ONLY_JWT_PROVIDER: CryptoProvider = CryptoProvider {
    signer_factory: hmac_jwt_signer_factory,
    verifier_factory: hmac_jwt_verifier_factory,
    jwk_utils: JwkUtils::new_unimplemented(),
};

#[derive(Clone)]
struct Hs256Signer(HmacSha256);

impl Hs256Signer {
    fn new(encoding_key: &EncodingKey) -> JwtResult<Self> {
        let inner = HmacSha256::new_from_slice(encoding_key.try_get_hmac_secret()?)
            .map_err(|_| new_error(JwtErrorKind::InvalidKeyFormat))?;
        Ok(Self(inner))
    }
}

impl Signer<Vec<u8>> for Hs256Signer {
    fn try_sign(&self, msg: &[u8]) -> std::result::Result<Vec<u8>, jsonwebtoken::signature::Error> {
        let mut signer = self.0.clone();
        signer.reset();
        signer.update(msg);
        Ok(signer.finalize().into_bytes().to_vec())
    }
}

impl JwtSigner for Hs256Signer {
    fn algorithm(&self) -> Algorithm {
        Algorithm::HS256
    }
}

#[derive(Clone)]
struct Hs256Verifier(HmacSha256);

impl Hs256Verifier {
    fn new(decoding_key: &DecodingKey) -> JwtResult<Self> {
        let inner = HmacSha256::new_from_slice(decoding_key.try_get_hmac_secret()?)
            .map_err(|_| new_error(JwtErrorKind::InvalidKeyFormat))?;
        Ok(Self(inner))
    }
}

impl Verifier<Vec<u8>> for Hs256Verifier {
    fn verify(
        &self,
        msg: &[u8],
        signature: &Vec<u8>,
    ) -> std::result::Result<(), jsonwebtoken::signature::Error> {
        let mut verifier = self.0.clone();
        verifier.reset();
        verifier.update(msg);
        verifier
            .verify_slice(signature)
            .map_err(jsonwebtoken::signature::Error::from_source)
    }
}

impl JwtVerifier for Hs256Verifier {
    fn algorithm(&self) -> Algorithm {
        Algorithm::HS256
    }
}

fn hmac_jwt_signer_factory(
    algorithm: &Algorithm,
    key: &EncodingKey,
) -> JwtResult<Box<dyn JwtSigner>> {
    match algorithm {
        Algorithm::HS256 => Ok(Box::new(Hs256Signer::new(key)?)),
        _ => Err(new_error(JwtErrorKind::InvalidAlgorithm)),
    }
}

fn hmac_jwt_verifier_factory(
    algorithm: &Algorithm,
    key: &DecodingKey,
) -> JwtResult<Box<dyn JwtVerifier>> {
    match algorithm {
        Algorithm::HS256 => Ok(Box::new(Hs256Verifier::new(key)?)),
        _ => Err(new_error(JwtErrorKind::InvalidAlgorithm)),
    }
}

pub(crate) fn ensure_jsonwebtoken_provider() {
    static INSTALL_ONCE: OnceLock<()> = OnceLock::new();
    INSTALL_ONCE.get_or_init(|| {
        let _ = HMAC_ONLY_JWT_PROVIDER.install_default();
    });
}

pub fn hash_gateway_api_key(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let digest = hasher.finalize();
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}

pub fn issue_jwt(subject: &str, role: AdminUserRole, signing_secret: &str) -> Result<String> {
    ensure_jsonwebtoken_provider();
    let issued_at = now_epoch_secs()?;
    let claims = Claims {
        sub: subject.to_owned(),
        role,
        iss: ADMIN_JWT_ISSUER.to_owned(),
        aud: ADMIN_JWT_AUDIENCE.to_owned(),
        exp: (issued_at + ADMIN_JWT_TTL_SECS) as usize,
        iat: issued_at as usize,
    };
    Ok(encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(signing_secret.as_bytes()),
    )?)
}

pub fn verify_jwt(token: &str, signing_secret: &str) -> Result<Claims> {
    ensure_jsonwebtoken_provider();
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&[ADMIN_JWT_AUDIENCE]);
    validation.set_issuer(&[ADMIN_JWT_ISSUER]);
    Ok(decode::<Claims>(
        token,
        &DecodingKey::from_secret(signing_secret.as_bytes()),
        &validation,
    )?
    .claims)
}

pub fn verify_portal_jwt(token: &str, signing_secret: &str) -> Result<PortalClaims> {
    ensure_jsonwebtoken_provider();
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&[PORTAL_JWT_AUDIENCE]);
    validation.set_issuer(&[PORTAL_JWT_ISSUER]);
    Ok(decode::<PortalClaims>(
        token,
        &DecodingKey::from_secret(signing_secret.as_bytes()),
        &validation,
    )?
    .claims)
}

