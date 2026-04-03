# Canonical Account Subject Resolution Design

**Goal:** add the clean-slate bridge from `GatewayAuthSubject` to the payable canonical `ai_account` so gateway billing can resolve exactly one active user account before hold or settlement logic is wired into the request path.

## Problem

The repository now has two working halves of the commercial kernel:

- `sdkwork-api-app-identity` can resolve a canonical `GatewayAuthSubject` from an API key
- `sdkwork-api-app-billing` can summarize account balances and plan holds for a known `account_id`

The missing seam is the business mapping between those two halves. Right now the system still lacks a standard way to answer:

- which `ai_account` is the payable account for this authenticated subject
- how the mapping behaves when the account is suspended or missing
- where that lookup lives so gateway, admin, jobs, and reconciliation do not duplicate the rule

## Approaches

### 1. Resolve the account inside the HTTP gateway

Keep account lookup logic in `sdkwork-api-interface-http` and filter account records there.

Pros:

- quick to wire for one request path

Cons:

- duplicates business rules in the wrong layer
- forces future admin, portal, and async-job paths to copy the same logic
- makes the plugin-first architecture weaker

### 2. Add an app-billing resolver on top of account-kernel storage

Keep the business rule in `sdkwork-api-app-billing`, but give it a storage seam that can find an account by canonical owner scope.

Pros:

- keeps gateway thin
- preserves one reusable commercial billing boundary
- aligns with the approved clean-slate account design

Cons:

- requires a new store method and new tests

### 3. Reuse `list_account_records()` and filter in memory

Avoid new storage APIs for now and scan all accounts in app-billing.

Pros:

- smallest immediate code change

Cons:

- wrong hot-path behavior for a production gateway
- weak contract for future database implementations
- hides an important indexed lookup behind a full table scan

## Recommendation

Choose approach 2.

This is the smallest correct commercial-grade step. The storage seam should expose owner-scope lookup, and `sdkwork-api-app-billing` should own the payable-account resolution rule.

## Design

### Storage seam

Add a targeted `AccountKernelStore` lookup:

- `find_account_record_by_owner(tenant_id, organization_id, user_id, account_type)`

This matches the canonical unique key already implied by the schema:

- `(tenant_id, organization_id, user_id, account_type)`

SQLite should implement the real indexed query now. Other dialects can continue to return explicit unsupported errors until their canonical account-kernel CRUD is real.

### Business rule

Add an app-billing function:

- `resolve_payable_account_for_gateway_subject(store, subject)`

Resolution rules:

1. lookup `account_type = primary`
2. match exact `tenant_id + organization_id + user_id`
3. return `None` if no primary account exists
4. reject non-`active` primary accounts with an error instead of silently charging them

This matches the clean-slate architecture decision that one user has one primary payable account.

### Why fail on inactive accounts

A suspended or closed account is not the same thing as “no account yet”.

For a commercial gateway, silently treating inactive accounts as absent would blur:

- operator misconfiguration
- compliance suspension
- user lifecycle state

The resolver should therefore fail closed on inactive accounts so the later gateway cutover can surface a deterministic admission failure.

### Scope of this slice

This slice does **not** add:

- transaction boundaries
- hold mutation
- settlement mutation
- gateway cutover

It only establishes the missing subject-to-account lookup seam so those next phases can be wired correctly.

## Testing

Drive the slice with TDD:

- SQLite-backed test proving owner-scope lookup resolves the right account
- app-billing test proving a `GatewayAuthSubject` resolves to an active primary account
- app-billing test proving a suspended primary account fails closed
- app-billing test proving missing account returns `None`

## Decision

Land canonical payable-account resolution now. It is the last missing bridge before transaction-safe hold and settlement orchestration can be implemented without leaking business rules into the HTTP layer.
