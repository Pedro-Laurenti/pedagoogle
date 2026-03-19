"use client";
import { useState, useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { convertFileSrc } from "@tauri-apps/api/core";
import { invokeCmd } from "@/utils/tauri";
import Toast from "@/components/Toast";
import type { Configuracoes, ToastState, MolduraEstilo } from "@/types";

interface ConfigForm {
  nome_escola: string;
  logo_path: string;
  cidade: string;
  diretor: string;
  moldura_estilo: MolduraEstilo;
  margem_folha: number;
  margem_moldura: number;
  margem_conteudo: number;
  [k: string]: unknown;
}

const MOLDURA_ESTILOS: { value: MolduraEstilo; label: string; desc: string }[] = [
  { value: 'none', label: 'Sem moldura', desc: 'Documento limpo sem bordas' },
  { value: 'simple', label: 'Simples', desc: 'Linha fina contínua' },
  { value: 'double', label: 'Dupla', desc: 'Duas linhas paralelas' },
  { value: 'ornate', label: 'Ornamentada', desc: 'Cantos decorativos clássicos' },
  { value: 'classic', label: 'Clássica', desc: 'Bordas com serifa elegante' },
  { value: 'modern', label: 'Moderna', desc: 'Linhas geométricas modernas' },
];

const EMPTY: ConfigForm = {
  nome_escola: "",
  logo_path: "",
  cidade: "",
  diretor: "",
  moldura_estilo: "none",
  margem_folha: 15,
  margem_moldura: 5,
  margem_conteudo: 5,
};

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
        molduraEstilo: form.moldura_estilo,
        margemFolha: form.margem_folha,
        margemMoldura: form.margem_moldura,
        margemConteudo: form.margem_conteudo,
      });
      notify("Configurações salvas.");
    } catch {
      notify("Erro ao salvar.", "error");
    }
  }

  return (
    <div className="max-w-3xl">
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

        <div className="divider">Layout de Exportação (PDF/Word)</div>

        <fieldset className="fieldset">
          <legend className="fieldset-legend">Estilo de Moldura</legend>
          <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
            {MOLDURA_ESTILOS.map((m) => (
              <label
                key={m.value}
                className={`cursor-pointer border rounded-lg p-3 transition-all ${
                  form.moldura_estilo === m.value
                    ? 'border-primary bg-primary/10'
                    : 'border-base-300 hover:border-base-content/30'
                }`}
              >
                <input
                  type="radio"
                  name="moldura"
                  value={m.value}
                  checked={form.moldura_estilo === m.value}
                  onChange={(e) => setForm({ ...form, moldura_estilo: e.target.value as MolduraEstilo })}
                  className="sr-only"
                />
                <div className="font-medium text-sm">{m.label}</div>
                <div className="text-xs opacity-60 mt-0.5">{m.desc}</div>
              </label>
            ))}
          </div>
        </fieldset>

        <fieldset className="fieldset">
          <legend className="fieldset-legend">Margens (em milímetros)</legend>
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <div>
              <label className="label text-xs">Margem da Folha</label>
              <input
                type="number"
                className="input w-full"
                value={form.margem_folha}
                onChange={(e) => setForm({ ...form, margem_folha: parseFloat(e.target.value) || 0 })}
                min={5}
                max={50}
                step={1}
              />
              <p className="label mt-1 text-xs opacity-60">Distância da borda do papel</p>
            </div>
            <div>
              <label className="label text-xs">Margem da Moldura</label>
              <input
                type="number"
                className="input w-full"
                value={form.margem_moldura}
                onChange={(e) => setForm({ ...form, margem_moldura: parseFloat(e.target.value) || 0 })}
                min={0}
                max={30}
                step={1}
              />
              <p className="label mt-1 text-xs opacity-60">Espaço até a moldura</p>
            </div>
            <div>
              <label className="label text-xs">Margem do Conteúdo</label>
              <input
                type="number"
                className="input w-full"
                value={form.margem_conteudo}
                onChange={(e) => setForm({ ...form, margem_conteudo: parseFloat(e.target.value) || 0 })}
                min={0}
                max={30}
                step={1}
              />
              <p className="label mt-1 text-xs opacity-60">Espaço após a moldura</p>
            </div>
          </div>
        </fieldset>

        {/* Preview da moldura */}
        {form.moldura_estilo !== 'none' && (
          <fieldset className="fieldset">
            <legend className="fieldset-legend">Pré-visualização</legend>
            <div className="bg-white rounded-lg p-4 flex justify-center">
              <MolduraPreview estilo={form.moldura_estilo} />
            </div>
          </fieldset>
        )}

        <div className="pt-2">
          <button type="submit" className="btn btn-primary">Salvar Configurações</button>
        </div>
      </form>
      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
    </div>
  );
}

// Componente de preview da moldura
function MolduraPreview({ estilo }: { estilo: MolduraEstilo }) {
  const w = 120;
  const h = 160;
  const m = 8; // margin for preview
  
  const renderFrame = () => {
    switch (estilo) {
      case 'simple':
        return <rect x={m} y={m} width={w - 2*m} height={h - 2*m} fill="none" stroke="#333" strokeWidth="1" />;
      case 'double':
        return (
          <>
            <rect x={m} y={m} width={w - 2*m} height={h - 2*m} fill="none" stroke="#333" strokeWidth="1" />
            <rect x={m+3} y={m+3} width={w - 2*m - 6} height={h - 2*m - 6} fill="none" stroke="#333" strokeWidth="1" />
          </>
        );
      case 'ornate':
        return (
          <>
            <rect x={m} y={m} width={w - 2*m} height={h - 2*m} fill="none" stroke="#333" strokeWidth="1" />
            {/* Corner ornaments */}
            <path d={`M${m},${m+10} Q${m+5},${m} ${m+10},${m}`} fill="none" stroke="#333" strokeWidth="1.5" />
            <path d={`M${w-m},${m+10} Q${w-m-5},${m} ${w-m-10},${m}`} fill="none" stroke="#333" strokeWidth="1.5" />
            <path d={`M${m},${h-m-10} Q${m+5},${h-m} ${m+10},${h-m}`} fill="none" stroke="#333" strokeWidth="1.5" />
            <path d={`M${w-m},${h-m-10} Q${w-m-5},${h-m} ${w-m-10},${h-m}`} fill="none" stroke="#333" strokeWidth="1.5" />
          </>
        );
      case 'classic':
        return (
          <>
            <rect x={m} y={m} width={w - 2*m} height={h - 2*m} fill="none" stroke="#333" strokeWidth="2" />
            <rect x={m+4} y={m+4} width={w - 2*m - 8} height={h - 2*m - 8} fill="none" stroke="#333" strokeWidth="0.5" />
            {/* Decorative lines in corners */}
            <line x1={m} y1={m+15} x2={m+15} y2={m} stroke="#333" strokeWidth="1" />
            <line x1={w-m} y1={m+15} x2={w-m-15} y2={m} stroke="#333" strokeWidth="1" />
            <line x1={m} y1={h-m-15} x2={m+15} y2={h-m} stroke="#333" strokeWidth="1" />
            <line x1={w-m} y1={h-m-15} x2={w-m-15} y2={h-m} stroke="#333" strokeWidth="1" />
          </>
        );
      case 'modern':
        return (
          <>
            {/* Top left corner block */}
            <rect x={m} y={m} width={20} height={3} fill="#333" />
            <rect x={m} y={m} width={3} height={20} fill="#333" />
            {/* Top right corner block */}
            <rect x={w-m-20} y={m} width={20} height={3} fill="#333" />
            <rect x={w-m-3} y={m} width={3} height={20} fill="#333" />
            {/* Bottom left corner block */}
            <rect x={m} y={h-m-3} width={20} height={3} fill="#333" />
            <rect x={m} y={h-m-20} width={3} height={20} fill="#333" />
            {/* Bottom right corner block */}
            <rect x={w-m-20} y={h-m-3} width={20} height={3} fill="#333" />
            <rect x={w-m-3} y={h-m-20} width={3} height={20} fill="#333" />
          </>
        );
      default:
        return null;
    }
  };

  return (
    <svg width={w} height={h} className="border border-base-300 bg-white">
      {renderFrame()}
      {/* Content placeholder lines */}
      <rect x={25} y={30} width={w-50} height={4} fill="#ddd" rx={2} />
      <rect x={25} y={40} width={w-50} height={4} fill="#ddd" rx={2} />
      <rect x={25} y={50} width={w-70} height={4} fill="#ddd" rx={2} />
      <rect x={25} y={70} width={w-50} height={4} fill="#ddd" rx={2} />
      <rect x={25} y={80} width={w-60} height={4} fill="#ddd" rx={2} />
    </svg>
  );
}
