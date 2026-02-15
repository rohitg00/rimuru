import { ReactNode, forwardRef } from "react";
import { AlertCircle, Info, CheckCircle } from "lucide-react";
import clsx from "clsx";
import styles from "./FormField.module.css";

interface FormFieldProps {
  label?: string;
  error?: string;
  hint?: string;
  success?: string;
  required?: boolean;
  className?: string;
  children: ReactNode;
}

export function FormField({
  label,
  error,
  hint,
  success,
  required,
  className,
  children,
}: FormFieldProps) {
  const hasError = !!error;
  const hasSuccess = !!success && !hasError;

  return (
    <div
      className={clsx(
        styles.field,
        hasError && styles.hasError,
        hasSuccess && styles.hasSuccess,
        className
      )}
    >
      {label && (
        <label className={styles.label}>
          {label}
          {required && <span className={styles.required}>*</span>}
        </label>
      )}
      {children}
      {(error || hint || success) && (
        <div
          className={clsx(
            styles.message,
            hasError && styles.error,
            hasSuccess && styles.success,
            !hasError && !hasSuccess && styles.hint
          )}
        >
          {hasError && <AlertCircle size={14} />}
          {hasSuccess && <CheckCircle size={14} />}
          {!hasError && !hasSuccess && hint && <Info size={14} />}
          <span>{error || success || hint}</span>
        </div>
      )}
    </div>
  );
}

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  error?: string;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ className, error, ...props }, ref) => {
    return (
      <input
        ref={ref}
        className={clsx(styles.input, error && styles.inputError, className)}
        aria-invalid={!!error}
        {...props}
      />
    );
  }
);
Input.displayName = "Input";

interface SelectProps extends React.SelectHTMLAttributes<HTMLSelectElement> {
  error?: string;
  options: { value: string; label: string }[];
  placeholder?: string;
}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(
  ({ className, error, options, placeholder, ...props }, ref) => {
    return (
      <select
        ref={ref}
        className={clsx(styles.select, error && styles.selectError, className)}
        aria-invalid={!!error}
        {...props}
      >
        {placeholder && (
          <option value="" disabled>
            {placeholder}
          </option>
        )}
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
    );
  }
);
Select.displayName = "Select";

interface TextAreaProps
  extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  error?: string;
}

export const TextArea = forwardRef<HTMLTextAreaElement, TextAreaProps>(
  ({ className, error, ...props }, ref) => {
    return (
      <textarea
        ref={ref}
        className={clsx(
          styles.textarea,
          error && styles.textareaError,
          className
        )}
        aria-invalid={!!error}
        {...props}
      />
    );
  }
);
TextArea.displayName = "TextArea";

interface CheckboxProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label: string;
  error?: string;
}

export const Checkbox = forwardRef<HTMLInputElement, CheckboxProps>(
  ({ className, label, error, ...props }, ref) => {
    return (
      <label className={clsx(styles.checkboxLabel, className)}>
        <input
          ref={ref}
          type="checkbox"
          className={clsx(styles.checkbox, error && styles.checkboxError)}
          aria-invalid={!!error}
          {...props}
        />
        <span className={styles.checkboxText}>{label}</span>
      </label>
    );
  }
);
Checkbox.displayName = "Checkbox";

interface FormActionsProps {
  children: ReactNode;
  className?: string;
}

export function FormActions({ children, className }: FormActionsProps) {
  return <div className={clsx(styles.actions, className)}>{children}</div>;
}

interface FormErrorProps {
  error?: string | null;
  className?: string;
}

export function FormError({ error, className }: FormErrorProps) {
  if (!error) return null;

  return (
    <div className={clsx(styles.formError, className)}>
      <AlertCircle size={18} />
      <span>{error}</span>
    </div>
  );
}
