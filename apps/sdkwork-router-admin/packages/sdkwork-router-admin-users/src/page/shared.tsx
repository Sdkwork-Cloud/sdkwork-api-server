import type { ReactNode } from 'react';
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@sdkwork/ui-pc-react';
import { translateAdminText } from 'sdkwork-router-admin-core';
import type { AdminPageProps, ManagedUser } from 'sdkwork-router-admin-types';

type UsersSnapshot = AdminPageProps['snapshot'];

export type SaveOperatorUserInput = {
  id?: string;
  email: string;
  display_name: string;
  password?: string;
  active: boolean;
};

export type SavePortalUserInput = {
  id?: string;
  email: string;
  display_name: string;
  password?: string;
  workspace_tenant_id: string;
  workspace_project_id: string;
  active: boolean;
};

export type OperatorDraft = {
  id?: string;
  email: string;
  display_name: string;
  password: string;
  active: boolean;
};

export type PortalDraft = {
  id?: string;
  email: string;
  display_name: string;
  password: string;
  workspace_tenant_id: string;
  workspace_project_id: string;
  active: boolean;
};

export type PendingDelete =
  | { kind: 'operator'; user: ManagedUser }
  | { kind: 'portal'; user: ManagedUser }
  | null;

export function defaultTenantId(snapshot: UsersSnapshot): string {
  return snapshot.tenants[0]?.id ?? 'tenant_local_demo';
}

export function defaultProjectId(
  snapshot: UsersSnapshot,
  tenantId: string,
): string {
  return snapshot.projects.find((project) => project.tenant_id === tenantId)?.id ?? '';
}

export function emptyOperatorDraft(): OperatorDraft {
  return {
    email: '',
    display_name: '',
    password: '',
    active: true,
  };
}

export function emptyPortalDraft(snapshot: UsersSnapshot): PortalDraft {
  const tenantId = defaultTenantId(snapshot);

  return {
    email: '',
    display_name: '',
    password: '',
    workspace_tenant_id: tenantId,
    workspace_project_id: defaultProjectId(snapshot, tenantId),
    active: true,
  };
}

export function operatorDraftFromUser(user: ManagedUser): OperatorDraft {
  return {
    id: user.id,
    email: user.email,
    display_name: user.display_name,
    password: '',
    active: user.active,
  };
}

export function portalDraftFromUser(
  user: ManagedUser,
  snapshot: UsersSnapshot,
): PortalDraft {
  const tenantId = user.workspace_tenant_id ?? defaultTenantId(snapshot);

  return {
    id: user.id,
    email: user.email,
    display_name: user.display_name,
    password: '',
    workspace_tenant_id: tenantId,
    workspace_project_id:
      user.workspace_project_id ?? defaultProjectId(snapshot, tenantId),
    active: user.active,
  };
}

export function matchesFilters(
  user: ManagedUser,
  deferredQuery: string,
  roleFilter: 'all' | 'operator' | 'portal',
  statusFilter: 'all' | 'active' | 'disabled',
): boolean {
  const roleMatches = roleFilter === 'all' || user.role === roleFilter;
  const statusMatches =
    statusFilter === 'all'
    || (statusFilter === 'active' && user.active)
    || (statusFilter === 'disabled' && !user.active);

  if (!roleMatches || !statusMatches) {
    return false;
  }

  const haystack = [
    user.display_name,
    user.email,
    user.workspace_tenant_id ?? '',
    user.workspace_project_id ?? '',
  ]
    .join(' ')
    .toLowerCase();

  return haystack.includes(deferredQuery);
}

export function isProtectedUser(
  user: ManagedUser,
  sessionUserId: string | null,
): boolean {
  return user.role === 'operator' && user.id === sessionUserId;
}

export function DialogField({
  children,
  description,
  htmlFor,
  label,
}: {
  children: ReactNode;
  description?: ReactNode;
  htmlFor?: string;
  label: ReactNode;
}) {
  return (
    <div className="space-y-2">
      <Label htmlFor={htmlFor}>{label}</Label>
      {children}
      {description ? (
        <div className="text-xs text-[var(--sdk-color-text-secondary)]">
          {description}
        </div>
      ) : null}
    </div>
  );
}

export function SelectField<T extends string>({
  description,
  disabled,
  label,
  labelVisibility = 'visible',
  onValueChange,
  options,
  placeholder,
  value,
}: {
  description?: ReactNode;
  disabled?: boolean;
  label: ReactNode;
  labelVisibility?: 'visible' | 'sr-only';
  onValueChange: (value: T) => void;
  options: Array<{ label: ReactNode; value: T }>;
  placeholder?: string;
  value: T;
}) {
  const isHiddenLabel = labelVisibility === 'sr-only';

  return (
    <div className={isHiddenLabel ? 'space-y-0' : 'space-y-2'}>
      <Label className={isHiddenLabel ? 'sr-only' : undefined}>{label}</Label>
      <Select
        disabled={disabled}
        onValueChange={(nextValue: string) => onValueChange(nextValue as T)}
        value={value}
      >
        <SelectTrigger>
          <SelectValue placeholder={placeholder ?? String(label)} />
        </SelectTrigger>
        <SelectContent>
          {options.map((option) => (
            <SelectItem key={option.value} value={option.value}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      {description ? (
        <div className="text-xs text-[var(--sdk-color-text-secondary)]">
          {description}
        </div>
      ) : null}
    </div>
  );
}

export function ConfirmActionDialog({
  confirmLabel = translateAdminText('Confirm'),
  description,
  onConfirm,
  onOpenChange,
  open,
  title,
}: {
  confirmLabel?: string;
  description: ReactNode;
  onConfirm: () => void | Promise<void>;
  onOpenChange: (open: boolean) => void;
  open: boolean;
  title: ReactNode;
}) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(92vw,28rem)]">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button onClick={() => onOpenChange(false)} type="button" variant="outline">
            {translateAdminText('Cancel')}
          </Button>
          <Button onClick={() => void onConfirm()} type="button" variant="danger">
            {confirmLabel}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
