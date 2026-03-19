import {
  createPortalApiKey,
  deletePortalApiKey,
  listPortalApiKeys,
  updatePortalApiKeyStatus,
} from 'sdkwork-router-portal-portal-api';
import type { CreatedGatewayApiKey, GatewayApiKeyRecord } from 'sdkwork-router-portal-types';

export function loadPortalApiKeys(): Promise<GatewayApiKeyRecord[]> {
  return listPortalApiKeys();
}

export function issuePortalApiKey(input: {
  environment: string;
  label: string;
  api_key?: string | null;
  notes?: string | null;
  expires_at_ms?: number | null;
}): Promise<CreatedGatewayApiKey> {
  return createPortalApiKey(input);
}

export function setPortalApiKeyActive(
  hashedKey: string,
  active: boolean,
): Promise<GatewayApiKeyRecord> {
  return updatePortalApiKeyStatus(hashedKey, active);
}

export function removePortalApiKey(hashedKey: string): Promise<void> {
  return deletePortalApiKey(hashedKey);
}
