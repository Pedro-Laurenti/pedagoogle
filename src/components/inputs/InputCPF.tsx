"use client";

interface Props {
  value: string;
  onChange: (v: string) => void;
  label?: string;
  required?: boolean;
  error?: string;
}

function aplicarMascara(v: string): string {
  const d = v.replace(/\D/g, "").slice(0, 11);
  if (d.length <= 3) return d;
  if (d.length <= 6) return `${d.slice(0, 3)}.${d.slice(3)}`;
  if (d.length <= 9) return `${d.slice(0, 3)}.${d.slice(3, 6)}.${d.slice(6)}`;
  return `${d.slice(0, 3)}.${d.slice(3, 6)}.${d.slice(6, 9)}-${d.slice(9)}`;
}

export default function InputCPF({ value, onChange, label, required, error }: Props) {
  return (
    <fieldset className="fieldset">
      {label && <legend className="fieldset-legend">{label}</legend>}
      <input
        className={`input w-full${error ? " input-error" : ""}`}
        type="text"
        value={value}
        onChange={(e) => onChange(aplicarMascara(e.target.value))}
        placeholder="000.000.000-00"
        required={required}
      />
      {error && <p className="text-error text-sm mt-1">{error}</p>}
    </fieldset>
  );
}
