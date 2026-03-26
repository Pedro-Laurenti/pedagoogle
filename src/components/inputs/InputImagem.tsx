"use client";
import { open } from "@tauri-apps/plugin-dialog";
import { copyFile, mkdir, readFile } from "@tauri-apps/plugin-fs";
import { appDataDir, join } from "@tauri-apps/api/path";
import { MdImage, MdDelete } from "react-icons/md";
import { useState, useEffect } from "react";

interface Props {
  value: string;
  onChange: (path: string) => void;
  label?: string;
}

export default function InputImagem({ value, onChange, label }: Props) {
  const [previewSrc, setPreviewSrc] = useState<string>("");

  useEffect(() => {
    if (!value) { setPreviewSrc(""); return; }
    readFile(value).then((bytes) => {
      const ext = value.split(".").pop()?.toLowerCase() ?? "png";
      const mime = ext === "jpg" || ext === "jpeg" ? "image/jpeg"
        : ext === "gif" ? "image/gif"
        : ext === "webp" ? "image/webp"
        : "image/png";
      const b64 = btoa(String.fromCharCode(...bytes));
      setPreviewSrc(`data:${mime};base64,${b64}`);
    }).catch(() => setPreviewSrc(""));
  }, [value]);

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
        {previewSrc && (
          <img
            src={previewSrc}
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
