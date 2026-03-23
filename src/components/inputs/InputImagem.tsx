"use client";
import { open } from "@tauri-apps/plugin-dialog";
import { copyFile, mkdir } from "@tauri-apps/plugin-fs";
import { appDataDir, join } from "@tauri-apps/api/path";
import { convertFileSrc } from "@tauri-apps/api/core";
import { MdImage, MdDelete } from "react-icons/md";

interface Props {
  value: string;
  onChange: (path: string) => void;
  label?: string;
}

export default function InputImagem({ value, onChange, label }: Props) {
  async function handleSelecionar() {
    const filePath = await open({
      filters: [{ name: "Imagem", extensions: ["png", "jpg", "jpeg", "gif", "webp"] }],
      multiple: false,
    });
    if (!filePath || typeof filePath !== "string") return;
    const dataDir = await appDataDir();
    const dir = await join(dataDir, "images");
    await mkdir(dir, { recursive: true });
    const filename = filePath.split(/[\\/]/).pop()!;
    const dest = await join(dir, filename);
    await copyFile(filePath, dest);
    onChange(dest);
  }

  return (
    <fieldset className="fieldset">
      {label && <legend className="fieldset-legend">{label}</legend>}
      <div className="flex flex-col gap-2">
        {value && (
          <img
            src={convertFileSrc(value)}
            alt="Imagem selecionada"
            className="max-h-40 rounded-box object-contain border border-base-300"
          />
        )}
        <div className="flex gap-2">
          <button type="button" className="btn btn-outline btn-sm gap-1" onClick={handleSelecionar}>
            <MdImage size={16} />
            {value ? "Alterar" : "Selecionar"}
          </button>
          {value && (
            <button type="button" className="btn btn-ghost btn-sm text-error gap-1" onClick={() => onChange("")}>
              <MdDelete size={16} />
              Remover
            </button>
          )}
        </div>
      </div>
    </fieldset>
  );
}
