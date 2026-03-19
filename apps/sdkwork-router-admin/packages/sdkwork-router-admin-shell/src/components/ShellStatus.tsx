import { ShieldCheck } from 'lucide-react';

function compactStatusLabel(status: string) {
  const normalized = status.toLowerCase();

  if (normalized.includes('synchronized')) {
    return 'Live sync';
  }

  if (normalized.includes('refresh')) {
    return 'Refreshing';
  }

  if (normalized.includes('authenticate')) {
    return 'Awaiting sign-in';
  }

  return status;
}

export function ShellStatus({ status }: { status: string }) {
  return (
    <div className="adminx-shell-meta-pill adminx-shell-meta-pill-status" title={status}>
      <ShieldCheck className="adminx-shell-meta-icon" />
      <span>{compactStatusLabel(status)}</span>
    </div>
  );
}
