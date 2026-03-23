"use client";
import { useState, useRef, useEffect } from "react";
import { MdClose, MdArrowDropDown } from "react-icons/md";

interface Opcao {
  value: string | number;
  label: string;
}

interface Props {
  options: Opcao[];
  value: (string | number)[];
  onChange: (v: (string | number)[]) => void;
  label?: string;
  placeholder?: string;
}

export default function InputMultiSelect({ options, value, onChange, label, placeholder = "Selecionar..." }: Props) {
  const [aberto, setAberto] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handler(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) setAberto(false);
    }
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  function alternar(v: string | number) {
    onChange(value.includes(v) ? value.filter((x) => x !== v) : [...value, v]);
  }

  const selecionados = options.filter((o) => value.includes(o.value));

  return (
    <fieldset className="fieldset">
      {label && <legend className="fieldset-legend">{label}</legend>}
      <div ref={ref} className="relative">
        <div
          className="input w-full flex items-center flex-wrap gap-1 cursor-pointer min-h-10 h-auto py-1"
          onClick={() => setAberto(!aberto)}
        >
          {selecionados.length === 0 && (
            <span className="text-base-content/40 text-sm flex-1">{placeholder}</span>
          )}
          {selecionados.map((o) => (
            <span key={o.value} className="badge badge-neutral gap-1">
              {o.label}
              <button
                type="button"
                className="hover:text-error"
                onClick={(e) => { e.stopPropagation(); alternar(o.value); }}
              >
                <MdClose size={12} />
              </button>
            </span>
          ))}
          <MdArrowDropDown className="ml-auto shrink-0" />
        </div>
        {aberto && (
          <ul className="absolute z-50 bg-base-100 border border-base-300 rounded-box shadow-lg w-full max-h-48 overflow-y-auto mt-1">
            {options.map((o) => (
              <li
                key={o.value}
                className="px-3 py-2 cursor-pointer hover:bg-base-200 flex items-center gap-2"
                onClick={() => alternar(o.value)}
              >
                <input
                  type="checkbox"
                  className="checkbox checkbox-sm"
                  checked={value.includes(o.value)}
                  readOnly
                />
                {o.label}
              </li>
            ))}
          </ul>
        )}
      </div>
    </fieldset>
  );
}
