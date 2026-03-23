"use client";
import { useState, useEffect } from "react";

interface Props {
  value: string;
  onChange: (v: string) => void;
  label?: string;
  required?: boolean;
  min?: string;
  max?: string;
}

function isoParaDisplay(iso: string): string {
  if (!iso || !/^\d{4}-\d{2}-\d{2}$/.test(iso)) return "";
  return `${iso.slice(8, 10)}/${iso.slice(5, 7)}/${iso.slice(0, 4)}`;
}

function displayParaISO(display: string): string {
  const d = display.replace(/\D/g, "");
  if (d.length !== 8) return "";
  return `${d.slice(4)}-${d.slice(2, 4)}-${d.slice(0, 2)}`;
}

function aplicarMascara(v: string): string {
  const d = v.replace(/\D/g, "").slice(0, 8);
  if (d.length <= 2) return d;
  if (d.length <= 4) return `${d.slice(0, 2)}/${d.slice(2)}`;
  return `${d.slice(0, 2)}/${d.slice(2, 4)}/${d.slice(4)}`;
}

export default function InputData({ value, onChange, label, required }: Props) {
  const [display, setDisplay] = useState(isoParaDisplay(value));

  useEffect(() => {
    setDisplay(isoParaDisplay(value));
  }, [value]);

  function handleChange(raw: string) {
    const mascarado = aplicarMascara(raw);
    setDisplay(mascarado);
    const iso = displayParaISO(mascarado);
    if (iso) onChange(iso);
    else if (!mascarado) onChange("");
  }

  return (
    <fieldset className="fieldset">
      {label && <legend className="fieldset-legend">{label}</legend>}
      <input
        className="input w-full"
        type="text"
        inputMode="numeric"
        value={display}
        onChange={(e) => handleChange(e.target.value)}
        placeholder="dd/mm/aaaa"
        required={required}
      />
    </fieldset>
  );
}
