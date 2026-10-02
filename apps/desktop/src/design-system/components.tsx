import type {
  ButtonHTMLAttributes,
  HTMLAttributes,
  PropsWithChildren,
  ReactNode,
} from "react";

export type AuraStatus = "Idle" | "Listening" | "Thinking" | "Working" | "Waiting";

type NavItemProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  active?: boolean;
  icon?: string;
};

export function AuraMark({ compact = false }: { compact?: boolean }) {
  return (
    <div className={compact ? "ds-aura-mark compact" : "ds-aura-mark"} aria-label="AURA">
      <img src="/aura-mark.svg" alt="" aria-hidden="true" />
    </div>
  );
}

export function NavItem({
  active = false,
  icon,
  children,
  className = "",
  ...props
}: NavItemProps) {
  return (
    <button
      className={`ds-nav-item ${active ? "active" : ""} ${className}`.trim()}
      {...props}
    >
      {icon ? (
        <span className="ds-nav-icon" aria-hidden="true">
          {icon}
        </span>
      ) : null}
      <span>{children}</span>
    </button>
  );
}

export function StatusPill({ status }: { status: AuraStatus }) {
  return (
    <div className="ds-status-pill" aria-label={`AURA status: ${status}`}>
      <span className={`ds-status-dot ${status.toLowerCase()}`} />
      <span>{status}</span>
    </div>
  );
}

export function Surface({
  children,
  className = "",
  ...props
}: PropsWithChildren<HTMLAttributes<HTMLElement>>) {
  return (
    <article className={`ds-surface ${className}`.trim()} {...props}>
      {children}
    </article>
  );
}

export function SectionLabel({
  children,
  trailing,
}: PropsWithChildren<{ trailing?: ReactNode }>) {
  return (
    <div className="ds-section-label">
      <span>{children}</span>
      {trailing}
    </div>
  );
}

export function ShortcutKey({ children }: PropsWithChildren) {
  return <kbd className="ds-key">{children}</kbd>;
}
