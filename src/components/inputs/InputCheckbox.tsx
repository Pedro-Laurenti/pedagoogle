"use client";

interface Props {
  checked: boolean;
  onChange: (v: boolean) => void;
  label: string;
  disabled?: boolean;
}

export default function InputCheckbox({ checked, onChange, label, disabled }: Props) {
  return (
    <fieldset className="fieldset">
      <label className="fieldset-label flex items-center gap-2 cursor-pointer">
        <input
          type="checkbox"
          className="checkbox"
          checked={checked}
          onChange={(e) => onChange(e.target.checked)}
          disabled={disabled}
        />
        <span>{label}</span>
      </label>
    </fieldset>
  );
}
