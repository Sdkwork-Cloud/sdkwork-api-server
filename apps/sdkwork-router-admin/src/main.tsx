import React from 'react';
import ReactDOM from 'react-dom/client';
import { bootstrapShellRuntime } from 'sdkwork-router-admin-shell';

import { App } from './App';

async function mountApp() {
  await bootstrapShellRuntime();

  ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
}

void mountApp();
