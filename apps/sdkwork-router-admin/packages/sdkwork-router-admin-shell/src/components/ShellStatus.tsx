import { ShieldCheck } from 'lucide-react';

export function ShellStatus({ status }: { status: string }) {
  return (
    <div className="adminx-shell-meta-pill adminx-shell-meta-pill-status" title={status}>
      <ShieldCheck className="adminx-shell-meta-icon" />
      <span>{status}</span>
    </div>
  );
}
