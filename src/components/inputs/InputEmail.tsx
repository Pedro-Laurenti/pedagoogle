"use client";

interface Props {
  value: string;
  onChange: (v: string) => void;
  label?: string;
  required?: boolean;
  error?: string;
}

export default function InputEmail({ value, onChange, label, required, error }: Props) {
  return (
    <fieldset className="fieldset">
      {label && <legend className="fieldset-legend">{label}</legend>}
      <input
        className={`input w-full${error ? " input-error" : ""}`}
        type="email"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder="exemplo@email.com"
        required={required}
      />
      {error && <p className="text-error text-sm mt-1">{error}</p>}
    </fieldset>
  );
}
