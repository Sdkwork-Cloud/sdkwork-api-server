import React from 'react';
import ReactDOM from 'react-dom/client';
import { preloadPreferredPortalLocale } from 'sdkwork-router-portal-commons';

import '@sdkwork/ui-pc-react/styles.css';
import './theme.css';

import { App } from './App';

async function bootstrapPortalApp() {
  try {
    await preloadPreferredPortalLocale();
  } finally {
    ReactDOM.createRoot(document.getElementById('root')!).render(
      <React.StrictMode>
        <App />
      </React.StrictMode>,
    );
  }
}

void bootstrapPortalApp();
