import { useState, useCallback, useMemo } from "react";

export type ValidationRule<T> = {
  validate: (value: T, allValues?: Record<string, unknown>) => boolean;
  message: string;
};

export type FieldValidation<T> = ValidationRule<T>[];

export interface FieldState<T> {
  value: T;
  error: string | null;
  touched: boolean;
  dirty: boolean;
}

export interface FormState<T extends Record<string, unknown>> {
  values: T;
  errors: Partial<Record<keyof T, string>>;
  touched: Partial<Record<keyof T, boolean>>;
  dirty: Partial<Record<keyof T, boolean>>;
  isValid: boolean;
  isSubmitting: boolean;
}

export interface UseFormOptions<T extends Record<string, unknown>> {
  initialValues: T;
  validations?: Partial<Record<keyof T, FieldValidation<T[keyof T]>>>;
  onSubmit?: (values: T) => Promise<void> | void;
  validateOnBlur?: boolean;
  validateOnChange?: boolean;
}

export function useForm<T extends Record<string, unknown>>({
  initialValues,
  validations = {},
  onSubmit,
  validateOnBlur = true,
  validateOnChange = false,
}: UseFormOptions<T>) {
  const [values, setValues] = useState<T>(initialValues);
  const [errors, setErrors] = useState<Partial<Record<keyof T, string>>>({});
  const [touched, setTouched] = useState<Partial<Record<keyof T, boolean>>>({});
  const [dirty, setDirty] = useState<Partial<Record<keyof T, boolean>>>({});
  const [isSubmitting, setIsSubmitting] = useState(false);

  const validateField = useCallback(
    (name: keyof T, value: T[keyof T]): string | null => {
      const fieldValidations = validations[name];
      if (!fieldValidations) return null;

      for (const rule of fieldValidations) {
        if (!rule.validate(value, values as Record<string, unknown>)) {
          return rule.message;
        }
      }
      return null;
    },
    [validations, values]
  );

  const validateAll = useCallback((): boolean => {
    const newErrors: Partial<Record<keyof T, string>> = {};
    let isValid = true;

    for (const key of Object.keys(values) as (keyof T)[]) {
      const error = validateField(key, values[key]);
      if (error) {
        newErrors[key] = error;
        isValid = false;
      }
    }

    setErrors(newErrors);
    return isValid;
  }, [values, validateField]);

  const handleChange = useCallback(
    (name: keyof T, value: T[keyof T]) => {
      setValues((prev) => ({ ...prev, [name]: value }));
      setDirty((prev) => ({ ...prev, [name]: true }));

      if (validateOnChange) {
        const error = validateField(name, value);
        setErrors((prev) => ({ ...prev, [name]: error ?? undefined }));
      }
    },
    [validateField, validateOnChange]
  );

  const handleBlur = useCallback(
    (name: keyof T) => {
      setTouched((prev) => ({ ...prev, [name]: true }));

      if (validateOnBlur) {
        const error = validateField(name, values[name]);
        setErrors((prev) => ({ ...prev, [name]: error ?? undefined }));
      }
    },
    [validateField, validateOnBlur, values]
  );

  const handleSubmit = useCallback(
    async (e?: React.FormEvent) => {
      e?.preventDefault();

      const allTouched: Partial<Record<keyof T, boolean>> = {};
      for (const key of Object.keys(values) as (keyof T)[]) {
        allTouched[key] = true;
      }
      setTouched(allTouched);

      const isValid = validateAll();
      if (!isValid || !onSubmit) return;

      setIsSubmitting(true);
      try {
        await onSubmit(values);
      } finally {
        setIsSubmitting(false);
      }
    },
    [values, validateAll, onSubmit]
  );

  const reset = useCallback(() => {
    setValues(initialValues);
    setErrors({});
    setTouched({});
    setDirty({});
    setIsSubmitting(false);
  }, [initialValues]);

  const setFieldValue = useCallback(
    (name: keyof T, value: T[keyof T]) => {
      handleChange(name, value);
    },
    [handleChange]
  );

  const setFieldError = useCallback((name: keyof T, error: string | null) => {
    setErrors((prev) => ({ ...prev, [name]: error ?? undefined }));
  }, []);

  const clearFieldError = useCallback((name: keyof T) => {
    setErrors((prev) => {
      const newErrors = { ...prev };
      delete newErrors[name];
      return newErrors;
    });
  }, []);

  const isValid = useMemo(() => {
    return Object.keys(errors).length === 0 && validateAll();
  }, [errors, validateAll]);

  const getFieldProps = useCallback(
    (name: keyof T) => ({
      value: values[name],
      onChange: (
        e: React.ChangeEvent<
          HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement
        >
      ) => handleChange(name, e.target.value as T[keyof T]),
      onBlur: () => handleBlur(name),
      error: touched[name] ? errors[name] : undefined,
      name: name as string,
    }),
    [values, errors, touched, handleChange, handleBlur]
  );

  return {
    values,
    errors,
    touched,
    dirty,
    isValid,
    isSubmitting,
    handleChange,
    handleBlur,
    handleSubmit,
    reset,
    setFieldValue,
    setFieldError,
    clearFieldError,
    validateField,
    validateAll,
    getFieldProps,
  };
}

export const validators = {
  required: <T>(message = "This field is required"): ValidationRule<T> => ({
    validate: (value) =>
      value !== null && value !== undefined && String(value).trim() !== "",
    message,
  }),

  minLength: (
    min: number,
    message?: string
  ): ValidationRule<string> => ({
    validate: (value) => !value || value.length >= min,
    message: message ?? `Must be at least ${min} characters`,
  }),

  maxLength: (
    max: number,
    message?: string
  ): ValidationRule<string> => ({
    validate: (value) => !value || value.length <= max,
    message: message ?? `Must be at most ${max} characters`,
  }),

  pattern: (
    regex: RegExp,
    message = "Invalid format"
  ): ValidationRule<string> => ({
    validate: (value) => !value || regex.test(value),
    message,
  }),

  email: (message = "Invalid email address"): ValidationRule<string> => ({
    validate: (value) =>
      !value || /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value),
    message,
  }),

  url: (message = "Invalid URL"): ValidationRule<string> => ({
    validate: (value) => {
      if (!value) return true;
      try {
        new URL(value);
        return true;
      } catch {
        return false;
      }
    },
    message,
  }),

  number: (message = "Must be a number"): ValidationRule<string> => ({
    validate: (value) => !value || !isNaN(Number(value)),
    message,
  }),

  min: (
    minValue: number,
    message?: string
  ): ValidationRule<number | string> => ({
    validate: (value) => {
      if (value === "" || value === null || value === undefined) return true;
      return Number(value) >= minValue;
    },
    message: message ?? `Must be at least ${minValue}`,
  }),

  max: (
    maxValue: number,
    message?: string
  ): ValidationRule<number | string> => ({
    validate: (value) => {
      if (value === "" || value === null || value === undefined) return true;
      return Number(value) <= maxValue;
    },
    message: message ?? `Must be at most ${maxValue}`,
  }),

  path: (message = "Invalid file path"): ValidationRule<string> => ({
    validate: (value) => {
      if (!value) return true;
      return /^(~|\/|\.\.?\/|[A-Za-z]:[\\/])/.test(value);
    },
    message,
  }),

  custom: <T>(
    fn: (value: T, allValues?: Record<string, unknown>) => boolean,
    message: string
  ): ValidationRule<T> => ({
    validate: fn,
    message,
  }),
};
