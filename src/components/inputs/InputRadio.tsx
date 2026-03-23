"use client";

interface Opcao {
  value: string;
  label: string;
}

interface Props {
  options: Opcao[];
  value: string;
  onChange: (v: string) => void;
  label?: string;
}

export default function InputRadio({ options, value, onChange, label }: Props) {
  return (
    <fieldset className="fieldset">
      {label && <legend className="fieldset-legend">{label}</legend>}
      <div className="flex flex-wrap gap-4">
        {options.map((opt) => (
          <label key={opt.value} className="flex items-center gap-2 cursor-pointer">
            <input
              type="radio"
              className="radio"
              value={opt.value}
              checked={value === opt.value}
              onChange={() => onChange(opt.value)}
            />
            <span>{opt.label}</span>
          </label>
        ))}
      </div>
    </fieldset>
  );
}
