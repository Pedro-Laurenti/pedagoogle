"use client";

interface Props {
  value: string;
  onChange: (v: string) => void;
  label?: string;
  required?: boolean;
}

export default function InputHora({ value, onChange, label, required }: Props) {
  return (
    <fieldset className="fieldset">
      {label && <legend className="fieldset-legend">{label}</legend>}
      <input
        className="input w-full"
        type="time"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        required={required}
      />
    </fieldset>
  );
}
