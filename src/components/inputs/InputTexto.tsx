"use client";

interface Props {
  value: string;
  onChange: (v: string) => void;
  label?: string;
  placeholder?: string;
  required?: boolean;
  error?: string;
  type?: string;
}

export default function InputTexto({ value, onChange, label, placeholder, required, error, type = "text" }: Props) {
  return (
    <fieldset className="fieldset">
      {label && <legend className="fieldset-legend">{label}</legend>}
      <input
        className={`input w-full${error ? " input-error" : ""}`}
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        required={required}
      />
      {error && <p className="text-error text-sm mt-1">{error}</p>}
    </fieldset>
  );
}
