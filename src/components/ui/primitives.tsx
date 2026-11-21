// Базовые UI-компоненты дизайн-системы Larpscord.

import { clsx } from "clsx";
import type { ReactNode, InputHTMLAttributes, TextareaHTMLAttributes } from "react";

export function Card({ className, children }: { className?: string; children: ReactNode }) {
  return <div className={clsx("glass rounded-[var(--radius-card)]", className)}>{children}</div>;
}

export function Section({
  title,
  desc,
  children,
  defaultOpen = true,
}: {
  title: string;
  desc?: string;
  children: ReactNode;
  defaultOpen?: boolean;
}) {
  return (
    <details className="editor-section group" open={defaultOpen}>
      <summary className="editor-section__head">
        <div>
          <div className="editor-section__title">{title}</div>
          {desc && <div className="editor-section__desc">{desc}</div>}
        </div>
        <svg className="editor-section__chev" width="12" height="12" viewBox="0 0 12 12" fill="none">
          <path d="M3 4.5L6 7.5L9 4.5" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      </summary>
      <div className="editor-section__body">{children}</div>
    </details>
  );
}

export function Field({
  label,
  hint,
  max,
  value,
  children,
}: {
  label: string;
  hint?: string;
  max?: number;
  value?: string;
  children: ReactNode;
}) {
  return (
    <label className="field">
      <div className="field__top">
        <span className="field__label">{label}</span>
        {max !== undefined && value !== undefined && (
          <span className={clsx("field__counter", value.length > max && "field__counter--over")}>
            {value.length}/{max}
          </span>
        )}
      </div>
      {children}
      {hint && <div className="field__hint">{hint}</div>}
    </label>
  );
}

export function TextInput(props: InputHTMLAttributes<HTMLInputElement>) {
  return <input className="input" {...props} />;
}

export function TextArea(props: TextareaHTMLAttributes<HTMLTextAreaElement>) {
  return <textarea className="input input--area" rows={2} {...props} />;
}

export function Toggle({
  checked,
  onChange,
  disabled,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      disabled={disabled}
      className={clsx("toggle", checked && "toggle--on")}
      onClick={() => onChange(!checked)}
    >
      <span className="toggle__thumb" />
    </button>
  );
}

export function Button({
  variant = "ghost",
  size = "md",
  className,
  children,
  ...rest
}: {
  variant?: "primary" | "ghost" | "danger" | "subtle";
  size?: "sm" | "md";
} & React.ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      className={clsx("btn", `btn--${variant}`, `btn--${size}`, className)}
      {...rest}
    >
      {children}
    </button>
  );
}

export function Badge({ tone, children }: { tone: "ok" | "warn" | "err" | "muted"; children: ReactNode }) {
  return <span className={clsx("badge", `badge--${tone}`)}>{children}</span>;
}
