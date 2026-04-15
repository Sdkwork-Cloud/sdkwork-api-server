use super::auth_utils::{
    enforce_gateway_request_rate_limit, extract_compat_gateway_token,
    resolve_authenticated_gateway_request_context,
};
use super::*;

#[derive(Clone, Debug)]
pub(crate) struct AuthenticatedGatewayRequest(IdentityGatewayRequestContext);

impl AuthenticatedGatewayRequest {
    pub(crate) fn tenant_id(&self) -> &str {
        self.0.tenant_id()
    }

    pub(crate) fn project_id(&self) -> &str {
        self.0.project_id()
    }

    pub(crate) fn context(&self) -> &IdentityGatewayRequestContext {
        &self.0
    }
}

impl FromRequestParts<GatewayApiState> for AuthenticatedGatewayRequest {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &GatewayApiState,
    ) -> Result<Self, Self::Rejection> {
        let context = if let Some(context) = parts
            .extensions
            .get::<IdentityGatewayRequestContext>()
            .cloned()
        {
            context
        } else {
            let Some(header_value) = parts.headers.get(header::AUTHORIZATION) else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };
            let Ok(header_value) = header_value.to_str() else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };
            let Some(token) = header_value
                .strip_prefix("Bearer ")
                .or_else(|| header_value.strip_prefix("bearer "))
            else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };

            let Some(context) = resolve_authenticated_gateway_request_context(state, token)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?
            else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };
            enforce_gateway_request_rate_limit(state.store.as_ref(), &context, parts.uri.path())
                .await?;
            context
        };

        Ok(Self(context))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CompatAuthenticatedGatewayRequest(IdentityGatewayRequestContext);

impl CompatAuthenticatedGatewayRequest {
    pub(crate) fn tenant_id(&self) -> &str {
        self.0.tenant_id()
    }

    pub(crate) fn project_id(&self) -> &str {
        self.0.project_id()
    }

    pub(crate) fn context(&self) -> &IdentityGatewayRequestContext {
        &self.0
    }
}

impl FromRequestParts<GatewayApiState> for CompatAuthenticatedGatewayRequest {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &GatewayApiState,
    ) -> Result<Self, Self::Rejection> {
        let context = if let Some(context) = parts
            .extensions
            .get::<IdentityGatewayRequestContext>()
            .cloned()
        {
            context
        } else {
            let Some(token) = extract_compat_gateway_token(parts) else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };

            let Some(context) = resolve_authenticated_gateway_request_context(state, &token)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?
            else {
                return Err(StatusCode::UNAUTHORIZED.into_response());
            };
            enforce_gateway_request_rate_limit(state.store.as_ref(), &context, parts.uri.path())
                .await?;
            context
        };

        Ok(Self(context))
    }
}
