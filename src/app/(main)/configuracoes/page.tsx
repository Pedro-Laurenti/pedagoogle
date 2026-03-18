"use client";
import { useState, useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { convertFileSrc } from "@tauri-apps/api/core";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import type { Configuracoes, ToastState } from "@/types";

interface ConfigForm { nome_escola: string; logo_path: string; cidade: string; diretor: string; [k: string]: unknown; }

const EMPTY: ConfigForm = { nome_escola: "", logo_path: "", cidade: "", diretor: "" };

export default function ConfiguracoesPage() {
  const [form, setForm] = useState<ConfigForm>(EMPTY);
  const [toast, setToast] = useState<ToastState | null>(null);

  useEffect(() => {
    invokeCmd<Configuracoes>("get_configuracoes").then((c) => setForm({ ...c })).catch(() => {});
  }, []);

  function notify(message: string, type: ToastState["type"] = "success") {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  }

  async function pickLogo() {
    const selected = await open({
      filters: [{ name: "Imagem", extensions: ["png", "jpg", "jpeg", "gif", "webp", "bmp"] }],
      multiple: false,
    });
    if (selected && typeof selected === "string") {
      setForm((f) => ({ ...f, logo_path: selected }));
    }
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      await invokeCmd("save_configuracoes", {
        nomeEscola: form.nome_escola,
        logoPath: form.logo_path,
        cidade: form.cidade,
        diretor: form.diretor,
      });
      notify("Configurações salvas.");
    } catch {
      notify("Erro ao salvar.", "error");
    }
  }

  return (
    <div className="max-w-2xl">
      <h1 className="text-3xl font-bold mb-6">Configurações da Escola</h1>
      <form onSubmit={handleSubmit} className="flex flex-col gap-4">
        <fieldset className="fieldset">
          <legend className="fieldset-legend">Nome da Escola</legend>
          <input
            className="input w-full"
            value={form.nome_escola}
            onChange={(e) => setForm({ ...form, nome_escola: e.target.value })}
            placeholder="Ex: Colégio Estadual São Paulo"
          />
        </fieldset>
        <fieldset className="fieldset">
          <legend className="fieldset-legend">Cidade</legend>
          <input
            className="input w-full"
            value={form.cidade}
            onChange={(e) => setForm({ ...form, cidade: e.target.value })}
            placeholder="Ex: São Paulo"
          />
        </fieldset>
        <fieldset className="fieldset">
          <legend className="fieldset-legend">Diretor(a)</legend>
          <input
            className="input w-full"
            value={form.diretor}
            onChange={(e) => setForm({ ...form, diretor: e.target.value })}
            placeholder="Nome completo"
          />
        </fieldset>
        <fieldset className="fieldset">
          <legend className="fieldset-legend">Logo da Escola</legend>
          <div className="flex gap-3 items-start">
            <div className="flex-1">
              <input
                className="input w-full"
                value={form.logo_path}
                onChange={(e) => setForm({ ...form, logo_path: e.target.value })}
                placeholder="Caminho para a imagem do logo"
              />
              <p className="label mt-1 text-xs opacity-60">PNG, JPG, GIF ou BMP — caminho absoluto no sistema</p>
            </div>
            <button type="button" className="btn btn-outline btn-sm mt-1" onClick={pickLogo}>
              Procurar…
            </button>
          </div>
          {form.logo_path && (
            <div className="mt-3 flex items-center gap-3">
              {/* eslint-disable-next-line @next/next/no-img-element */}
              <img
                src={convertFileSrc(form.logo_path)}
                alt="Logo"
                className="h-16 w-auto object-contain rounded border border-base-300"
                onError={(e) => { (e.target as HTMLImageElement).style.display = "none"; }}
              />
              <button
                type="button"
                className="btn btn-ghost btn-xs text-error"
                onClick={() => setForm({ ...form, logo_path: "" })}
              >
                Remover
              </button>
            </div>
          )}
        </fieldset>
        <div className="pt-2">
          <button type="submit" className="btn btn-primary">Salvar Configurações</button>
        </div>
      </form>
      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
    </div>
  );
}
