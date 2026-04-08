declare module 'react-router-dom' {
  import type { ComponentType, ReactElement, ReactNode } from 'react';

  export type NavigateFunction = (
    to: number | string,
    options?: { replace?: boolean },
  ) => void;

  export type NavLinkRenderProps = {
    isActive: boolean;
  };

  export type NavLinkProps = {
    children?: ReactNode | ((props: NavLinkRenderProps) => ReactNode);
    className?: string | ((props: NavLinkRenderProps) => string);
    onBlur?: () => void;
    onFocus?: () => void;
    onMouseEnter?: () => void;
    onMouseLeave?: () => void;
    onPointerDown?: () => void;
    title?: string;
    to: string;
    'data-slot'?: string;
  };

  export const BrowserRouter: ComponentType<{
    basename?: string;
    children?: ReactNode;
  }>;

  export const Navigate: ComponentType<{
    replace?: boolean;
    to: string;
  }>;

  export const Route: ComponentType<{
    element?: ReactNode;
    index?: boolean;
    path?: string;
  }>;

  export const Routes: ComponentType<{
    children?: ReactNode;
  }>;

  export function NavLink(props: NavLinkProps): ReactElement | null;
  export function useLocation(): {
    pathname: string;
    search: string;
  };
  export function useNavigate(): NavigateFunction;
  export function useSearchParams(): [
    URLSearchParams,
    (
      nextInit: Record<string, string> | URLSearchParams | string,
      options?: { replace?: boolean },
    ) => void,
  ];
}
