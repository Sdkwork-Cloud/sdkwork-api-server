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
        <div className="text-xs text-[var(--sdk-color-text-secondary)]">{description}</div>
      ) : null}
    </div>
  );
}

export function SelectField<T extends string>({
  description,
  disabled,
  label,
  onValueChange,
  options,
  placeholder,
  value,
}: {
  description?: ReactNode;
  disabled?: boolean;
  label: ReactNode;
  onValueChange: (value: T) => void;
  options: Array<{ label: ReactNode; value: T }>;
  placeholder?: string;
  value: T;
}) {
  return (
    <div className="space-y-2">
      <Label>{label}</Label>
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
        <div className="text-xs text-[var(--sdk-color-text-secondary)]">{description}</div>
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
      <DialogContent className="w-[min(92vw,30rem)]">
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

export function joinCommaSeparatedList(values: string[]): string {
  return values.join(', ');
}

export function parseCommaSeparatedList(value: string): string[] {
  return value
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean);
}

export function previewJson(value: string, maxLength = 96): string {
  const normalizedValue = value.trim().replace(/\s+/g, ' ');
  if (normalizedValue.length <= maxLength) {
    return normalizedValue || '{}';
  }

  return `${normalizedValue.slice(0, maxLength - 1)}...`;
}

export function createLocalId(prefix: string): string {
  return `${prefix}_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
}

export function resolveStatusVariant(
  status: string,
): 'secondary' | 'success' | 'warning' | 'danger' {
  const normalizedStatus = status.trim().toLowerCase();
  if (
    normalizedStatus.includes('success')
    || normalizedStatus.includes('settled')
    || normalizedStatus.includes('completed')
    || normalizedStatus.includes('processed')
    || normalizedStatus.includes('authorized')
    || normalizedStatus.includes('refunded')
    || normalizedStatus.includes('matched')
    || normalizedStatus.includes('active')
    || normalizedStatus.includes('enabled')
  ) {
    return 'success';
  }
  if (
    normalizedStatus.includes('fail')
    || normalizedStatus.includes('reject')
    || normalizedStatus.includes('expired')
    || normalizedStatus.includes('disabled')
    || normalizedStatus.includes('mismatch')
    || normalizedStatus.includes('dead_letter')
  ) {
    return 'danger';
  }
  if (
    normalizedStatus.includes('pending')
    || normalizedStatus.includes('retry')
    || normalizedStatus.includes('requires')
    || normalizedStatus.includes('warning')
    || normalizedStatus.includes('partial')
  ) {
    return 'warning';
  }
  return 'secondary';
}
