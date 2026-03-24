"use client";
import IMask from "imask";
import { IMaskInput } from "react-imask";

interface Props {
  value: string;
  onChange: (v: string) => void;
  label?: string;
  required?: boolean;
}

const HORA_BLOCKS = {
  HH: { mask: IMask.MaskedRange, from: 0, to: 23, maxLength: 2 },
  MM: { mask: IMask.MaskedRange, from: 0, to: 59, maxLength: 2 },
};

export default function InputHora({ value, onChange, label, required }: Props) {
  return (
    <fieldset className="fieldset">
      {label && <legend className="fieldset-legend">{label}</legend>}
      <IMaskInput
        className="input w-full"
        mask="HH:MM"
        blocks={HORA_BLOCKS}
        value={value}
        onAccept={(v) => onChange(String(v))}
        placeholder="00:00"
        required={required}
      />
    </fieldset>
  );
}
