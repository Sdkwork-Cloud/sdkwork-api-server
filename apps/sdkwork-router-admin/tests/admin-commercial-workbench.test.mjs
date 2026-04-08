import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

function readSourceTree(relativeDirectory) {
  const sourceRoot = path.join(appRoot, relativeDirectory);
  const chunks = [];

  function visit(directory) {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const fullPath = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        visit(fullPath);
        continue;
      }

      if (fullPath.endsWith('.ts') || fullPath.endsWith('.tsx')) {
        chunks.push(readFileSync(fullPath, 'utf8'));
      }
    }
  }

  visit(sourceRoot);
  return chunks.join('\n');
}

test('admin commercial workspace wires canonical billing investigation into types, API, workbench, and gateway pages', () => {
  const adminTypes = read('packages/sdkwork-router-admin-types/src/index.ts');
  const adminApi = readSourceTree('packages/sdkwork-router-admin-admin-api/src');
  const workbench = read('packages/sdkwork-router-admin-core/src/workbench.tsx');
  const snapshot = read('packages/sdkwork-router-admin-core/src/workbenchSnapshot.ts');
  const accessPage = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayAccessPage.tsx');
  const usagePage = read('packages/sdkwork-router-admin-apirouter/src/pages/GatewayUsagePage.tsx');
  const commercialSource = readSourceTree('packages/sdkwork-router-admin-commercial/src');

  assert.match(adminTypes, /export interface CommercialAccountRecord/);
  assert.match(adminTypes, /export interface CommercialAccountSummary/);
  assert.match(adminTypes, /export interface CommercialAccountBalanceSnapshot/);
  assert.match(adminTypes, /export interface CommercialAccountBenefitLotRecord/);
  assert.match(adminTypes, /export interface CommercialAccountHoldRecord/);
  assert.match(adminTypes, /export interface CommercialRequestSettlementRecord/);
  assert.match(adminTypes, /export type CommercialAccountLedgerEntryType =/);
  assert.match(adminTypes, /export interface CommercialAccountLedgerEntryRecord/);
  assert.match(adminTypes, /export interface CommercialAccountLedgerAllocationRecord/);
  assert.match(adminTypes, /export interface CommercialAccountLedgerHistoryEntry/);
  assert.match(adminTypes, /export interface CommerceOrderRecord/);
  assert.match(adminTypes, /export interface CommercePaymentEventRecord/);
  assert.match(adminTypes, /export interface CommerceOrderAuditRecord/);
  assert.match(adminTypes, /export interface CommercialPricingPlanRecord/);
  assert.match(adminTypes, /export interface CommercialPricingRateRecord/);
  assert.match(adminTypes, /commercialAccounts:/);
  assert.match(adminTypes, /commercialAccountHolds:/);
  assert.match(adminTypes, /commercialAccountLedger:/);
  assert.match(adminTypes, /commercialRequestSettlements:/);
  assert.match(adminTypes, /commerceOrders:/);
  assert.match(adminTypes, /commercePaymentEvents:/);
  assert.match(adminTypes, /commercialPricingPlans:/);
  assert.match(adminTypes, /commercialPricingRates:/);

  assert.match(adminApi, /listCommercialAccounts/);
  assert.match(adminApi, /getCommercialAccountBalance/);
  assert.match(adminApi, /listCommercialAccountBenefitLots/);
  assert.match(adminApi, /listCommercialAccountLedger/);
  assert.match(adminApi, /listCommercialAccountHolds/);
  assert.match(adminApi, /listCommercialRequestSettlements/);
  assert.match(adminApi, /listCommercialPricingPlans/);
  assert.match(adminApi, /listCommercialPricingRates/);
  assert.match(adminApi, /listRecentCommerceOrders/);
  assert.match(adminApi, /listCommercePaymentEvents/);
  assert.match(adminApi, /getCommerceOrderAudit/);

  assert.match(workbench, /listCommercialAccounts/);
  assert.match(workbench, /listCommercialAccountLedger/);
  assert.match(workbench, /listCommercialAccountHolds/);
  assert.match(workbench, /listCommercialRequestSettlements/);
  assert.match(workbench, /listCommercialPricingPlans/);
  assert.match(workbench, /listCommercialPricingRates/);
  assert.match(workbench, /listRecentCommerceOrders/);
  assert.match(workbench, /listCommercePaymentEvents/);
  assert.match(snapshot, /commercialAccounts:/);
  assert.match(snapshot, /commercialAccountHolds:/);
  assert.match(snapshot, /commercialAccountLedger:/);
  assert.match(snapshot, /commercialRequestSettlements:/);
  assert.match(snapshot, /commerceOrders:/);
  assert.match(snapshot, /commercePaymentEvents:/);
  assert.match(snapshot, /commercialPricingPlans:/);
  assert.match(snapshot, /commercialPricingRates:/);

  assert.match(accessPage, /Commercial governance/);
  assert.match(accessPage, /Commercial accounts/);
  assert.match(accessPage, /Pricing posture/);
  assert.match(usagePage, /Commercial accounts/);
  assert.match(usagePage, /Request settlements/);
  assert.match(usagePage, /Pricing posture/);

  assert.match(commercialSource, /Settlement ledger/);
  assert.match(commercialSource, /Refund timeline/);
  assert.match(commercialSource, /Order payment audit/);
  assert.match(commercialSource, /Order refund audit/);
  assert.match(commercialSource, /Order audit detail/);
  assert.match(commercialSource, /View order audit/);
  assert.match(commercialSource, /Find order audit/);
  assert.match(commercialSource, /normalizeCommercialOrderAuditLookupValue/);
  assert.match(commercialSource, /buildCommercialLedgerTimelineRows/);
  assert.match(commercialSource, /buildCommercialOrderPaymentAuditRows/);
  assert.match(commercialSource, /commercialAccountLedger/);
  assert.match(commercialSource, /commercePaymentEvents/);
});
