import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const appRoot = path.resolve(import.meta.dirname, '..');

function read(relativePath) {
  return readFileSync(path.join(appRoot, relativePath), 'utf8');
}

test('billing workspace localizes order lifecycle labels and dynamic checkout copy through shared portal i18n', () => {
  const commons = read('packages/sdkwork-router-portal-commons/src/index.tsx');
  const billingPage = read('packages/sdkwork-router-portal-billing/src/pages/index.tsx');

  assert.match(billingPage, /header:\s*t\('Offer'\)/);
  assert.match(billingPage, /header:\s*t\('Kind'\)/);
  assert.match(billingPage, /header:\s*t\('Coupon'\)/);
  assert.match(billingPage, /header:\s*t\('Payable'\)/);
  assert.match(billingPage, /header:\s*t\('Granted units'\)/);
  assert.match(billingPage, /header:\s*t\('Status'\)/);
  assert.match(billingPage, /header:\s*t\('Created'\)/);
  assert.match(billingPage, /header:\s*t\('Actions'\)/);
  assert.match(
    billingPage,
    /setCheckoutSessionStatus\(\s*t\('Open the checkout workbench from Pending payment queue to inspect the selected payment method\.'\),?\s*\)/,
  );
  assert.match(
    billingPage,
    /setCheckoutSessionStatus\(\s*t\('Loading checkout for \{orderId\}\.\.\.',[\s\S]*?\{\s*orderId\s*\}[\s\S]*?\)\s*\)/,
  );
  assert.match(
    billingPage,
    /setCheckoutStatus\(\s*t\('Loading live checkout pricing for \{targetId\}\.\.\.',[\s\S]*?\{\s*targetId:\s*selection\.target\.id[\s\S]*?\}\s*[\s\S]*?\)\s*\)/,
  );
  assert.match(
    billingPage,
    /setCheckoutStatus\(\s*t\('Creating a checkout order for \{targetId\}\.\.\.',[\s\S]*?\{\s*targetId:\s*checkoutSelection\.target\.id[\s\S]*?\}\s*[\s\S]*?\)\s*\)/,
  );
  assert.match(
    billingPage,
    /t\(\s*'\{targetName\} was queued in Pending payment queue\. Open the checkout workbench to complete payment before quota or membership changes are applied\.',[\s\S]*?\{\s*targetName:\s*order\.target_name[\s\S]*?\}\s*[\s\S]*?\)/,
  );
  assert.match(
    billingPage,
    /t\('Reopening \{provider\} checkout for \{targetName\}\.\.\.'/,
  );
  assert.match(
    billingPage,
    /t\('Starting \{provider\} checkout for \{targetName\}\.\.\.'/,
  );
  assert.match(
    billingPage,
    /t\('Creating a fresh \{provider\} checkout attempt for \{targetName\}\.\.\.'/,
  );
  assert.match(
    billingPage,
    /t\('\{targetName\} now uses the \{provider\} checkout launch path\.'/,
  );
  assert.match(
    billingPage,
    /t\('\{targetName\} created a \{provider\} checkout attempt, but no checkout link was returned\.'/,
  );
  assert.match(
    billingPage,
    /t\('\{reference\} is the current \{provider\} \/ \{channel\} payment reference for this order\.'/,
  );
  assert.match(
    billingPage,
    /t\('\{reference\} is the current checkout reference for this order\.'/,
  );
  assert.match(
    billingPage,
    /t\(\s*'Commercial account keeps balance, holds, and account identity visible beside the workspace billing posture\.'/,
  );
  assert.match(
    billingPage,
    /t\(\s*'Failed payment keeps checkout attempts that need coupon updates, a different payment method, or a fresh checkout visible for follow-up\.'/,
  );
  assert.match(
    billingPage,
    /t\(\s*'Billing view keeps live quota, checkout progress, and payment history in one place\.'/,
  );
  assert.match(billingPage, /function targetKindLabel\(/);
  assert.match(billingPage, /function orderStatusLabel\(/);
  assert.match(billingPage, /function checkoutSessionStatusLabel\(/);
  assert.match(billingPage, /function checkoutMethodActionLabel\(/);
  assert.match(billingPage, /function checkoutMethodAvailabilityLabel\(/);
  assert.match(billingPage, /targetKindLabel\(row\.target_kind,\s*t\)/);
  assert.match(billingPage, /orderStatusLabel\(row\.status,\s*t\)/);
  assert.match(billingPage, /checkoutSessionStatusLabel\(checkoutSession\.session_status,\s*t\)/);
  assert.match(billingPage, /checkoutMethodActionLabel\(method\.action,\s*t\)/);
  assert.match(billingPage, /checkoutMethodAvailabilityLabel\(method\.availability,\s*t\)/);
  assert.match(billingPage, /t\('\{kind\} \/ \{price\}',\s*\{\s*kind:/);
  assert.doesNotMatch(billingPage, /è·¯/);

  for (const key of [
    'Offer',
    'Kind',
    'Coupon',
    'Payable',
    'Payment pending',
    'Fulfilled',
    'Canceled',
    'Failed',
    'Open checkout',
    'Open the checkout workbench from Pending payment queue to inspect the selected payment method.',
    'Loading checkout for {orderId}...',
    'Loading live checkout pricing for {targetId}...',
    'Creating a checkout order for {targetId}...',
    '{targetName} was queued in Pending payment queue. Open the checkout workbench to complete payment before quota or membership changes are applied.',
    'Opening {provider} checkout for {targetName}...',
    'Starting {provider} checkout for {targetName}...',
    '{targetName} now uses the {provider} checkout launch path.',
    '{targetName} created a {provider} checkout attempt, but no checkout link was returned.',
    'Subscription plan',
    'Recharge pack',
    'Manual settlement',
    'Settle order',
    'Cancel order',
    'Checkout access',
    'Planned',
    'Closed',
    'Opening checkout...',
    'Open checkout link',
    'Start checkout',
    'Resume checkout',
    'Retry with new attempt',
    'The latest {provider} checkout can still be resumed, so the workbench will reopen the existing checkout.',
    'The latest {provider} attempt no longer has a reusable checkout link, so the workbench will create a fresh checkout attempt.',
    'No {provider} checkout attempt exists yet for this order. Start the first checkout now.',
    'Reopening {provider} checkout for {targetName}...',
    'Creating a fresh {provider} checkout attempt for {targetName}...',
    'Checkout attempts that closed on the failure path and need a fresh checkout decision.',
    '{reference} is the current {provider} / {channel} payment reference for this order.',
    '{reference} is the current checkout reference for this order.',
    '{kind} / {price}',
    '{planName} is the active workspace membership and defines the current subscription entitlement baseline.',
    'No active membership is recorded yet. Complete a subscription checkout to activate monthly entitlement posture.',
    'Pending payment queue keeps unpaid or unfulfilled orders visible until payment completes or the order leaves the queue.',
    'Failed payment keeps checkout attempts that need coupon updates, a different payment method, or a fresh checkout visible for follow-up.',
    'Billing view keeps live quota, checkout progress, and payment history in one place.',
    'Payment history keeps checkout outcomes, payment method evidence, and refund status visible in one billing timeline.',
    'Refund history keeps completed refund outcomes, payment method evidence, and the resulting order status visible without reopening each order.',
    'Settlement coverage',
    'Settlement coverage keeps benefit lots, credit holds, and request capture aligned in one billing snapshot.',
    'Fallback reasoning stays visible so you can distinguish degraded routing from the preferred routing path.',
    'Payment outcome sandbox',
    'Apply settlement, failure, or cancellation outcomes for the selected payment method before live payment confirmation is enabled.',
    'Sandbox only',
    'Sandbox method',
    'Choose sandbox method',
    'Payment outcomes will use {provider} on {channel}.',
    'Manual step',
    'Hosted checkout flow',
    'QR checkout flow',
    'Checkout flow',
    'Payment outcomes',
    'Verification method',
    'Checkout reference',
    'Manual confirmation',
    'Signed callback check',
    'Stripe signature check',
    'Alipay RSA-SHA256 check',
    'WeChat Pay RSA-SHA256 check',
    'Refund coverage',
    'Partial refunds',
    'QR code content',
    'Apply settlement outcome',
    'Apply failure outcome',
    'Apply cancellation outcome',
    'Applying {provider} settlement outcome for {orderId}...',
    'Applying {provider} failure outcome for {orderId}...',
    'Applying {provider} cancellation outcome for {orderId}...',
    'Commercial account keeps balance, holds, and account identity visible beside the workspace billing posture.',
    '{targetName} was settled after the {provider} payment confirmation.',
    '{targetName} was marked failed after the {provider} payment confirmation.',
    '{targetName} was canceled after the {provider} payment confirmation.',
    '{targetName} was settled after provider payment confirmation.',
    '{targetName} was canceled after provider payment confirmation.',
    'Open billing workbench',
    'Awaiting pending order',
    'Checkout attempts',
    'Checkout attempts keep checkout access, retries, and checkout references visible inside the same workbench.',
    'Latest attempt',
    'No checkout attempts recorded yet',
    'No checkout attempts have been recorded for this order yet.',
    'Primary method',
    'Payment method',
    'Payment update reference',
    'Checkout workbench keeps checkout access, selected reference, and payable price aligned under one payment method.',
    'Current status',
    'This checkout is already closed, so there are no remaining payment actions.',
    'No checkout guidance is available for this order yet.',
  ]) {
    assert.match(commons, new RegExp(`'${key.replace(/[.*+?^${}()|[\\]\\\\]/g, '\\\\$&')}'`));
  }

  assert.doesNotMatch(commons, /Commercial settlement rail/);
  assert.doesNotMatch(commons, /Provider callbacks/);
  assert.doesNotMatch(commons, /Provider webhooks/);
  assert.doesNotMatch(commons, /Callback rail/);
  assert.doesNotMatch(commons, /Choose provider rail/);
  assert.doesNotMatch(commons, /callback rehearsal/);
  assert.doesNotMatch(commons, /callback flow/);
  assert.doesNotMatch(commons, /provider callback/);
  assert.doesNotMatch(commons, /provider handoff/);
  assert.doesNotMatch(commons, /Provider events/);
  assert.doesNotMatch(commons, /Provider event/);
  assert.doesNotMatch(commons, /Launching provider checkout/);
  assert.doesNotMatch(commons, /Open provider checkout/);
  assert.doesNotMatch(commons, /Launch provider checkout/);
  assert.doesNotMatch(commons, /Resume provider checkout/);
  assert.doesNotMatch(commons, /Launch the first provider checkout now/);
  assert.doesNotMatch(commons, /payment attempt/);
  assert.doesNotMatch(commons, /payment attempts/);
  assert.doesNotMatch(commons, /Replay provider settlement, failure, or cancellation events/);
  assert.doesNotMatch(commons, /Replaying \{provider\} settlement/);
  assert.doesNotMatch(commons, /Replaying \{provider\} failure/);
  assert.doesNotMatch(commons, /Replaying \{provider\} cancellation/);
  assert.doesNotMatch(commons, /Replay settlement event/);
  assert.doesNotMatch(commons, /Replay failure event/);
  assert.doesNotMatch(commons, /Replay cancel event/);
  assert.doesNotMatch(commons, /Payment event sandbox/);
  assert.doesNotMatch(commons, /Event target/);
  assert.doesNotMatch(commons, /Choose event target/);
  assert.doesNotMatch(commons, /active sandbox target/);
  assert.doesNotMatch(commons, /Event signature/);
  assert.doesNotMatch(commons, /stripe_signature/);
  assert.doesNotMatch(commons, /alipay_rsa_sha256/);
  assert.doesNotMatch(commons, /wechatpay_rsa_sha256/);
  assert.doesNotMatch(commons, /webhook_signed/);
  assert.doesNotMatch(commons, /\bPayment rail\b/);
  assert.doesNotMatch(commons, /\bPrimary rail\b/);
  assert.doesNotMatch(commons, /\bEvent rail\b/);
  assert.doesNotMatch(commons, /selected payment rail/);
  assert.doesNotMatch(commons, /sandbox rail/);
  assert.doesNotMatch(commons, /Operator action/);
  assert.doesNotMatch(commons, /Webhook verification/);
  assert.doesNotMatch(commons, /\bWebhook\b/);
  assert.doesNotMatch(commons, /Open session/);
  assert.doesNotMatch(commons, /Loading checkout session for \{orderId\}\.\.\./);
  assert.doesNotMatch(commons, /existing provider session/);
  assert.doesNotMatch(commons, /Manual action/);
  assert.doesNotMatch(commons, /Hosted checkout session/);
  assert.doesNotMatch(commons, /QR code session/);
  assert.doesNotMatch(commons, /This checkout session is already closed, so there are no remaining payment actions\./);
  assert.doesNotMatch(commons, /Session reference/);
  assert.doesNotMatch(commons, /QR payload/);
  assert.doesNotMatch(commons, /Refund support/);
  assert.doesNotMatch(commons, /\bPartial refund\b/);
  assert.doesNotMatch(commons, /closed-loop refund outcomes/);
  assert.doesNotMatch(commons, /refund closure/);
  assert.doesNotMatch(commons, /operators can distinguish degraded routing from normal preference selection/);
  assert.doesNotMatch(
    commons,
    /Commercial account exposes canonical balance, hold, and account identity state beside the workspace billing posture\./,
  );
  assert.doesNotMatch(
    commons,
    /Failed payment isolates checkout attempts that need coupon updates, a different payment method, or a fresh checkout attempt\./,
  );
  assert.doesNotMatch(
    commons,
    /Billing posture now combines live quota evidence, checkout state, and the payment lifecycle timeline\./,
  );
  assert.doesNotMatch(
    commons,
    /\{reference\} anchors the current \{provider\} \/ \{channel\} payment method for this order\./,
  );
  assert.doesNotMatch(commons, /verify provider, checkout reference, and final order state without reopening each order/);
  assert.doesNotMatch(
    commons,
    /Formal order-scoped checkout attempts keep checkout access, retries, and checkout references visible inside the same workbench\./,
  );
  assert.doesNotMatch(
    commons,
    /Formal checkout keeps checkout access, selected reference, and payable price aligned under one payment method\./,
  );
  assert.doesNotMatch(commons, /No formal guidance is available for this order yet\./);
  assert.doesNotMatch(
    commons,
    /\{targetName\} created a formal \{provider\} checkout attempt, but no checkout URL was returned\./,
  );
  assert.doesNotMatch(
    commons,
    /\{targetName\} now uses the formal \{provider\} checkout launch path\./,
  );
  assert.doesNotMatch(
    billingPage,
    /Formal order-scoped checkout attempts keep checkout access, retries, and checkout references visible inside the same workbench\./,
  );
  assert.doesNotMatch(
    billingPage,
    /Commercial account exposes canonical balance, hold, and account identity state beside the workspace billing posture\./,
  );
  assert.doesNotMatch(
    billingPage,
    /Failed payment isolates checkout attempts that need coupon updates, a different payment method, or a fresh checkout attempt\./,
  );
  assert.doesNotMatch(
    billingPage,
    /Billing posture now combines live quota evidence, checkout state, and the payment lifecycle timeline\./,
  );
  assert.doesNotMatch(
    billingPage,
    /\{reference\} anchors the current \{provider\} \/ \{channel\} payment method for this order\./,
  );
  assert.doesNotMatch(
    billingPage,
    /Formal checkout keeps checkout access, selected reference, and payable price aligned under one payment method\./,
  );
  assert.doesNotMatch(billingPage, /No formal guidance is available for this order yet\./);
  assert.doesNotMatch(
    billingPage,
    /\{targetName\} created a formal \{provider\} checkout attempt, but no checkout URL was returned\./,
  );
  assert.doesNotMatch(
    billingPage,
    /\{targetName\} now uses the formal \{provider\} checkout launch path\./,
  );
});
