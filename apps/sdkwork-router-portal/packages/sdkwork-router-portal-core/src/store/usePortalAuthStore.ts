import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import {
  clearPortalSessionToken,
  getPortalDashboard,
  getPortalMe,
  getPortalWorkspace,
  loginPortalUser,
  onPortalSessionExpired,
  persistPortalSessionToken,
  portalErrorMessage,
  PortalApiError,
  readPortalSessionToken,
  registerPortalUser,
} from 'sdkwork-router-portal-portal-api';
import type {
  PortalAuthSession,
  PortalDashboardSummary,
  PortalUserProfile,
  PortalWorkspaceSummary,
} from 'sdkwork-router-portal-types';

const PORTAL_AUTH_STORAGE_KEY = 'sdkwork-router-portal.auth.v1';
const DEFAULT_BOOTSTRAP_STATUS = 'Checking for an existing portal session token.';
const DEFAULT_DASHBOARD_STATUS = 'Sign in to load the current workspace status.';

interface PortalAuthState {
  isAuthenticated: boolean;
  isBootstrapping: boolean;
  sessionToken: string | null;
  user: PortalUserProfile | null;
  workspace: PortalWorkspaceSummary | null;
  dashboardSnapshot: PortalDashboardSummary | null;
  bootstrapStatus: string;
  dashboardStatus: string;
  signIn: (credentials: { email: string; password: string }) => Promise<PortalAuthSession>;
  register: (payload: { name: string; email: string; password: string }) => Promise<PortalAuthSession>;
  signOut: (message?: string) => Promise<void>;
  hydrate: () => Promise<void>;
  syncWorkspace: (token?: string) => Promise<PortalWorkspaceSummary | null>;
  syncDashboard: (token?: string) => Promise<PortalDashboardSummary | null>;
}

function signedOutState(message = DEFAULT_BOOTSTRAP_STATUS) {
  return {
    isAuthenticated: false,
    isBootstrapping: false,
    sessionToken: null,
    user: null,
    workspace: null,
    dashboardSnapshot: null,
    bootstrapStatus: message,
    dashboardStatus: DEFAULT_DASHBOARD_STATUS,
  } as const;
}

async function syncSessionState(
  session: PortalAuthSession,
  get: () => PortalAuthState,
  set: (
    partial:
      | Partial<PortalAuthState>
      | ((state: PortalAuthState) => Partial<PortalAuthState>),
  ) => void,
): Promise<void> {
  persistPortalSessionToken(session.token);

  set({
    isAuthenticated: true,
    isBootstrapping: true,
    sessionToken: session.token,
    user: session.user,
    bootstrapStatus: 'Refreshing workspace identity and dashboard context.',
    dashboardStatus: 'Refreshing workspace status after sign-in.',
  });

  await get().syncWorkspace(session.token);
  await get().syncDashboard(session.token);

  set({
    isAuthenticated: true,
    isBootstrapping: false,
    sessionToken: session.token,
    user: session.user,
    bootstrapStatus: 'Workspace identity restored.',
  });
}

export const usePortalAuthStore = create<PortalAuthState>()(
  persist(
    (set, get) => ({
      ...signedOutState(),
      signIn: async (credentials) => {
        const session = await loginPortalUser(credentials);
        await syncSessionState(session, get, set);
        return session;
      },
      register: async (payload) => {
        const session = await registerPortalUser({
          display_name: payload.name,
          email: payload.email,
          password: payload.password,
        });
        await syncSessionState(session, get, set);
        return session;
      },
      signOut: async (message) => {
        clearPortalSessionToken();
        set(signedOutState(message ?? 'Your portal session ended. Sign in again to continue.'));
      },
      hydrate: async () => {
        const persistedToken = readPortalSessionToken();

        if (!persistedToken) {
          set(signedOutState());
          return;
        }

        set({
          isBootstrapping: true,
          bootstrapStatus: 'Refreshing workspace identity and navigation context.',
          dashboardStatus: 'Refreshing workspace status for the active project.',
          sessionToken: persistedToken,
        });

        try {
          const [user, workspace] = await Promise.all([
            getPortalMe(persistedToken),
            getPortalWorkspace(persistedToken),
          ]);

          set({
            isAuthenticated: true,
            isBootstrapping: true,
            sessionToken: persistedToken,
            user,
            workspace,
            bootstrapStatus: 'Workspace identity restored.',
          });

          await get().syncDashboard(persistedToken);

          set({
            isAuthenticated: true,
            isBootstrapping: false,
            sessionToken: persistedToken,
            user,
            workspace,
          });
        } catch (error) {
          const nextMessage =
            error instanceof PortalApiError && error.status === 401
              ? 'The saved portal session is no longer valid. Please sign in again.'
              : portalErrorMessage(error);

          clearPortalSessionToken();
          set(signedOutState(nextMessage));
        }
      },
      syncWorkspace: async (token) => {
        const currentToken = token ?? get().sessionToken ?? readPortalSessionToken();

        if (!currentToken) {
          set({ workspace: null });
          return null;
        }

        try {
          const workspace = await getPortalWorkspace(currentToken);
          set({
            isAuthenticated: true,
            sessionToken: currentToken,
            workspace,
          });
          return workspace;
        } catch (error) {
          if (error instanceof PortalApiError && error.status === 401) {
            clearPortalSessionToken();
            set(signedOutState('Your portal session expired. Sign in again to continue.'));
            return null;
          }

          set({
            bootstrapStatus: portalErrorMessage(error),
          });
          return null;
        }
      },
      syncDashboard: async (token) => {
        const currentToken = token ?? get().sessionToken ?? readPortalSessionToken();

        if (!currentToken) {
          set({
            dashboardSnapshot: null,
            dashboardStatus: DEFAULT_DASHBOARD_STATUS,
          });
          return null;
        }

        try {
          const dashboardSnapshot = await getPortalDashboard(currentToken);
          set({
            isAuthenticated: true,
            sessionToken: currentToken,
            dashboardSnapshot,
            dashboardStatus: 'Workspace status is synced with the latest dashboard snapshot.',
          });
          return dashboardSnapshot;
        } catch (error) {
          if (error instanceof PortalApiError && error.status === 401) {
            clearPortalSessionToken();
            set(signedOutState('Your portal session expired. Sign in again to continue.'));
            return null;
          }

          set({
            dashboardStatus: portalErrorMessage(error),
          });
          return null;
        }
      },
    }),
    {
      name: PORTAL_AUTH_STORAGE_KEY,
      partialize: (state) => ({
        isAuthenticated: state.isAuthenticated,
        sessionToken: state.sessionToken,
        user: state.user,
        workspace: state.workspace,
      }),
      merge: (persistedState, currentState) => ({
        ...currentState,
        ...(persistedState as Partial<PortalAuthState>),
        bootstrapStatus: currentState.bootstrapStatus,
        dashboardStatus: currentState.dashboardStatus,
        dashboardSnapshot: currentState.dashboardSnapshot,
      }),
    },
  ),
);

export function subscribeToPortalSessionExpiry() {
  return onPortalSessionExpired(() => {
    void usePortalAuthStore
      .getState()
      .signOut('Your portal session expired. Sign in again to continue.');
  });
}
