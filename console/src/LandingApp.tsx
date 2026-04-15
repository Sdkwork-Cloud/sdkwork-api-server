import './landing.css';

export function LandingApp() {
  const portalHref = import.meta.env.DEV ? 'http://127.0.0.1:5174/' : '/portal/';

  return (
    <main className="landing-app">
      <section className="landing-hero">
        <div className="landing-copy">
          <p className="landing-eyebrow">SDKWork Browser Entry</p>
          <h1>Two products. Two surfaces. Two security boundaries.</h1>
          <p className="landing-text">
            `admin` is the operator command center for routing, runtime, and tenant governance.
            `portal` is the developer-facing self-service product for workspace access and API key
            issuance.
          </p>
          <div className="landing-actions">
            <a className="landing-button landing-button-primary" href="/admin/">
              Open admin app
            </a>
            <a className="landing-button landing-button-secondary" href={portalHref}>
              Open portal app
            </a>
          </div>
        </div>
        <aside className="landing-callout">
          <span>Development identities come from the active bootstrap profile.</span>
          <span>Use the operator or portal email provisioned by your runtime configuration.</span>
          <span>Passwords stay in runtime configuration and are never prefilled in the UI.</span>
        </aside>
      </section>

      <section className="landing-grid">
        <article className="landing-card landing-card-admin">
          <p className="landing-card-label">Admin App</p>
          <h2>Operator control plane</h2>
          <p>
            Govern tenants, channels, routing policy, traffic cost, and extension runtimes from a
            dense operations-first surface.
          </p>
          <ul className="landing-list">
            <li>Operator-only authentication domain</li>
            <li>Runtime and routing observability</li>
            <li>Workspace and provider governance</li>
          </ul>
          <a className="landing-button landing-button-primary" href="/admin/">
            Launch admin
          </a>
        </article>

        <article className="landing-card landing-card-portal">
          <p className="landing-card-label">Portal App</p>
          <h2>Developer self-service workspace</h2>
          <p>
            Register, sign in, inspect workspace ownership, rotate portal credentials, and issue
            environment-scoped API keys without touching the control plane.
          </p>
          <ul className="landing-list">
            <li>Developer onboarding and workspace identity</li>
            <li>Self-serve API key lifecycle</li>
            <li>Portal-only account security</li>
          </ul>
          <a className="landing-button landing-button-secondary" href={portalHref}>
            Launch portal
          </a>
        </article>
      </section>
    </main>
  );
}
