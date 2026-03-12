import { appName } from 'sdkwork-api-core';
import { ChannelRegistryPage } from 'sdkwork-api-channel';
import { RouteSimulationPage } from 'sdkwork-api-routing';
import { RuntimeStatusPage } from 'sdkwork-api-runtime';
import { RequestExplorerPage } from 'sdkwork-api-usage';
import { WorkspaceDashboard } from 'sdkwork-api-workspace';

export function App() {
  return (
    <main>
      <h1>{appName}</h1>
      <WorkspaceDashboard />
      <ChannelRegistryPage />
      <RouteSimulationPage />
      <RequestExplorerPage />
      <RuntimeStatusPage />
    </main>
  );
}
