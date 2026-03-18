import {
  cloneElement,
  createContext,
  isValidElement,
  useContext,
  useEffect,
  useMemo,
  useState,
  type MouseEvent,
  type ReactElement,
  type ReactNode,
} from 'react';

function cx(...values: Array<string | false | null | undefined>): string {
  return values.filter(Boolean).join(' ');
}

export function SectionHero({
  eyebrow,
  title,
  detail,
  actions,
}: {
  eyebrow: string;
  title: string;
  detail: string;
  actions?: ReactNode;
}) {
  return (
    <div className="adminx-hero">
      <div>
        <p className="adminx-eyebrow">{eyebrow}</p>
        <h1>{title}</h1>
        <p className="adminx-hero-detail">{detail}</p>
      </div>
      {actions ? <div className="adminx-hero-actions">{actions}</div> : null}
    </div>
  );
}

export function PageToolbar({
  title,
  detail,
  actions,
  children,
}: {
  title: string;
  detail: string;
  actions?: ReactNode;
  children?: ReactNode;
}) {
  return (
    <section className="adminx-page-toolbar">
      <div className="adminx-page-toolbar-head">
        <div className="adminx-page-toolbar-copy">
          <h2>{title}</h2>
          <p>{detail}</p>
        </div>
        {actions ? <div className="adminx-page-toolbar-actions">{actions}</div> : null}
      </div>
      {children ? <div className="adminx-page-toolbar-body">{children}</div> : null}
    </section>
  );
}

export function Surface({
  title,
  detail,
  actions,
  children,
}: {
  title: string;
  detail?: string;
  actions?: ReactNode;
  children: ReactNode;
}) {
  return (
    <section className="adminx-surface">
      <div className="adminx-surface-heading">
        <div>
          <h2>{title}</h2>
          {detail ? <p>{detail}</p> : null}
        </div>
        {actions ? <div className="adminx-surface-actions">{actions}</div> : null}
      </div>
      {children}
    </section>
  );
}

export function StatCard({
  label,
  value,
  detail,
}: {
  label: string;
  value: string;
  detail: string;
}) {
  return (
    <article className="adminx-stat-card">
      <span>{label}</span>
      <strong>{value}</strong>
      <p>{detail}</p>
    </article>
  );
}

export function Pill({
  tone,
  children,
}: {
  tone?: 'default' | 'live' | 'seed' | 'danger';
  children: ReactNode;
}) {
  return <span className={`adminx-pill adminx-pill-${tone ?? 'default'}`}>{children}</span>;
}

export function DataTable<T>({
  columns,
  rows,
  empty,
  getKey,
}: {
  columns: Array<{ key: string; label: string; render: (row: T) => ReactNode }>;
  rows: T[];
  empty: string;
  getKey: (row: T, index: number) => string;
}) {
  return (
    <div className="adminx-table-wrap">
      <table className="adminx-table">
        <thead>
          <tr>
            {columns.map((column) => (
              <th key={column.key}>{column.label}</th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row, index) => (
            <tr key={getKey(row, index)} className="adminx-table-row">
              {columns.map((column) => (
                <td key={column.key}>{column.render(row)}</td>
              ))}
            </tr>
          ))}
          {!rows.length ? (
            <tr>
              <td colSpan={columns.length} className="adminx-empty">
                {empty}
              </td>
            </tr>
          ) : null}
        </tbody>
      </table>
    </div>
  );
}

export function InlineButton({
  children,
  onClick,
  tone,
  type,
  disabled,
  className = '',
}: {
  children: ReactNode;
  onClick?: () => void;
  tone?: 'primary' | 'secondary' | 'danger';
  type?: 'button' | 'submit';
  disabled?: boolean;
  className?: string;
}) {
  return (
    <button
      className={`adminx-button adminx-button-${tone ?? 'secondary'} ${className}`.trim()}
      disabled={disabled}
      onClick={onClick}
      type={type ?? 'button'}
    >
      {children}
    </button>
  );
}

type DialogContextValue = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
};

const DialogContext = createContext<DialogContextValue | null>(null);

function useDialogContext(): DialogContextValue {
  const context = useContext(DialogContext);

  if (!context) {
    throw new Error('Dialog primitives must be used inside Dialog.');
  }

  return context;
}

function composeClickHandlers(
  original: ((event: MouseEvent<HTMLElement>) => void) | undefined,
  next: (event: MouseEvent<HTMLElement>) => void,
) {
  return (event: MouseEvent<HTMLElement>) => {
    original?.(event);
    if (!event.defaultPrevented) {
      next(event);
    }
  };
}

export function Dialog({
  open,
  onOpenChange,
  children,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  children: ReactNode;
}) {
  const value = useMemo(
    () => ({ open, onOpenChange }),
    [open, onOpenChange],
  );

  return <DialogContext.Provider value={value}>{children}</DialogContext.Provider>;
}

export function DialogTrigger({
  asChild,
  children,
}: {
  asChild?: boolean;
  children: ReactNode;
}) {
  const { onOpenChange } = useDialogContext();

  if (asChild && isValidElement(children)) {
    const element = children as ReactElement<{ onClick?: (event: MouseEvent<HTMLElement>) => void }>;
    return cloneElement(element, {
      onClick: composeClickHandlers(element.props.onClick, () => onOpenChange(true)),
    });
  }

  return (
    <button
      type="button"
      className="adminx-button adminx-button-secondary"
      onClick={() => onOpenChange(true)}
    >
      {children}
    </button>
  );
}

export function DialogContent({
  children,
  size = 'medium',
}: {
  children: ReactNode;
  size?: 'small' | 'medium' | 'large';
}) {
  const { open, onOpenChange } = useDialogContext();

  useEffect(() => {
    if (!open) {
      return;
    }

    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = 'hidden';

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onOpenChange(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);

    return () => {
      document.body.style.overflow = previousOverflow;
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [onOpenChange, open]);

  if (!open) {
    return null;
  }

  return (
    <div
      className="adminx-dialog-backdrop"
      role="presentation"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) {
          onOpenChange(false);
        }
      }}
    >
      <div
        className={cx('adminx-dialog-panel', `adminx-dialog-panel-${size}`)}
        role="dialog"
        aria-modal="true"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <button
          type="button"
          className="adminx-dialog-close"
          onClick={() => onOpenChange(false)}
          aria-label="Close dialog"
        >
          x
        </button>
        {children}
      </div>
    </div>
  );
}

export function DialogHeader({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return <div className={cx('adminx-dialog-header', className)}>{children}</div>;
}

export function DialogTitle({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return <strong className={cx('adminx-dialog-title', className)}>{children}</strong>;
}

export function DialogDescription({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return <p className={cx('adminx-dialog-description', className)}>{children}</p>;
}

export function DialogFooter({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return <div className={cx('adminx-dialog-actions', className)}>{children}</div>;
}

export function AdminDialog({
  title,
  detail,
  actions,
  children,
}: {
  title: string;
  detail?: string;
  actions?: ReactNode;
  children: ReactNode;
}) {
  return (
    <>
      <DialogHeader>
        <div className="adminx-dialog-copy">
          <DialogTitle>{title}</DialogTitle>
          {detail ? <DialogDescription>{detail}</DialogDescription> : null}
        </div>
      </DialogHeader>
      <div className="adminx-dialog-body">{children}</div>
      {actions ? <DialogFooter>{actions}</DialogFooter> : null}
    </>
  );
}

export function FormField({
  label,
  children,
  hint,
  className,
}: {
  label: string;
  children: ReactNode;
  hint?: string;
  className?: string;
}) {
  return (
    <label className={cx('adminx-field', className)}>
      <span>{label}</span>
      {children}
      {hint ? <small className="adminx-field-hint">{hint}</small> : null}
    </label>
  );
}

export function ConfirmDialog({
  open,
  title,
  detail,
  confirmLabel,
  onClose,
  onConfirm,
}: {
  open: boolean;
  title: string;
  detail: string;
  confirmLabel: string;
  onClose: () => void;
  onConfirm: () => Promise<void> | void;
}) {
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    if (!open) {
      setIsSubmitting(false);
    }
  }, [open]);

  return (
    <Dialog
      open={open}
      onOpenChange={(nextOpen) => {
        if (!nextOpen && !isSubmitting) {
          onClose();
        }
      }}
    >
      <DialogContent size="small">
        <AdminDialog
          title={title}
          detail={detail}
          actions={(
            <>
              <InlineButton disabled={isSubmitting} onClick={onClose}>
                Cancel
              </InlineButton>
              <InlineButton
                tone="danger"
                disabled={isSubmitting}
                onClick={() => {
                  setIsSubmitting(true);
                  void Promise.resolve(onConfirm()).finally(() => {
                    setIsSubmitting(false);
                  });
                }}
              >
                {isSubmitting ? 'Working...' : confirmLabel}
              </InlineButton>
            </>
          )}
        >
          <div className="adminx-confirm-dialog">
            <p>{detail}</p>
          </div>
        </AdminDialog>
      </DialogContent>
    </Dialog>
  );
}
