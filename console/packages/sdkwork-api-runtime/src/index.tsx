import type { RuntimeMode } from 'sdkwork-api-types';

const activeMode: RuntimeMode = 'embedded';

export function RuntimeStatusPage() {
  return (
    <section>
      <h2>Runtime Status</h2>
      <p>Current host mode: {activeMode}</p>
    </section>
  );
}
