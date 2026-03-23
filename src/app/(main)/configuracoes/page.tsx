"use client";
import { useState, useEffect } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
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
  fonte: string;
  nota_minima: number;
  ano_letivo: string;
  tamanho_fonte: number;
  tema: string;
  usar_turmas: boolean;
  usar_professores: boolean;
  usar_frequencia: boolean;
  usar_recuperacao: boolean;
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
  fonte: "New Computer Modern",
  nota_minima: 5,
  ano_letivo: "2026",
  tamanho_fonte: 11,
  tema: "light",
  usar_turmas: true,
  usar_professores: true,
  usar_frequencia: true,
  usar_recuperacao: true,
};

export default function ConfiguracoesPage() {
  const [form, setForm] = useState<ConfigForm>(EMPTY);
  const [toast, setToast] = useState<ToastState | null>(null);

  useEffect(() => {
    invokeCmd<Configuracoes>("get_configuracoes").then((c) => {
      setForm({ ...EMPTY, ...c });
      document.documentElement.setAttribute("data-theme", c.tema || "light");
    }).catch(() => {});
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
        fonte: form.fonte,
        notaMinima: form.nota_minima,
        anoLetivo: form.ano_letivo,
        tamanhoFonte: form.tamanho_fonte,
        tema: form.tema,
        usarTurmas: form.usar_turmas,
        usarProfessores: form.usar_professores,
        usarFrequencia: form.usar_frequencia,
        usarRecuperacao: form.usar_recuperacao,
      });
      document.documentElement.setAttribute("data-theme", form.tema);
      notify("Configurações salvas.");
    } catch {
      notify("Erro ao salvar.", "error");
    }
  }

  async function handleBackup() {
    const filePath = await save({ defaultPath: "pedagoogle_backup.db", filters: [{ name: "SQLite", extensions: ["db"] }] });
    if (!filePath) return;
    try {
      await invokeCmd("backup_database", { path: filePath });
      notify("Backup exportado com sucesso.");
    } catch {
      notify("Erro ao exportar backup.", "error");
    }
  }

  async function handleRestore() {
    const filePath = await open({ filters: [{ name: "SQLite", extensions: ["db"] }], multiple: false });
    if (!filePath || typeof filePath !== "string") return;
    if (!window.confirm("Restaurar banco de dados? Os dados atuais serão substituídos.")) return;
    try {
      await invokeCmd("restore_database", { path: filePath });
      notify("Banco restaurado. Reinicie o aplicativo para aplicar as alterações.");
    } catch {
      notify("Erro ao restaurar backup.", "error");
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

        <fieldset className="fieldset">
          <legend className="fieldset-legend">Fonte</legend>
          <select
            className="select w-full"
            value={form.fonte}
            onChange={(e) => setForm({ ...form, fonte: e.target.value })}
          >
            <option value="New Computer Modern">New Computer Modern</option>
            <option value="DejaVu Sans">DejaVu Sans</option>
            <option value="Libertinus Serif">Libertinus Serif</option>
          </select>
        </fieldset>

        <fieldset className="fieldset">
          <legend className="fieldset-legend">Tamanho da fonte PDF (pt)</legend>
          <input
            type="number"
            className="input w-full"
            value={form.tamanho_fonte}
            onChange={(e) => setForm({ ...form, tamanho_fonte: parseInt(e.target.value) || 11 })}
            min={8}
            max={16}
          />
        </fieldset>

        <div className="divider">Avaliação</div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <fieldset className="fieldset">
            <legend className="fieldset-legend">Nota mínima de aprovação</legend>
            <input
              type="number"
              className="input w-full"
              value={form.nota_minima}
              onChange={(e) => setForm({ ...form, nota_minima: parseFloat(e.target.value) || 0 })}
              min={0}
              max={10}
              step={0.5}
            />
          </fieldset>
          <fieldset className="fieldset">
            <legend className="fieldset-legend">Ano letivo</legend>
            <input
              className="input w-full"
              value={form.ano_letivo}
              onChange={(e) => setForm({ ...form, ano_letivo: e.target.value })}
              placeholder="Ex: 2026"
            />
          </fieldset>
        </div>

        <div className="divider">Aparência</div>

        <fieldset className="fieldset">
          <legend className="fieldset-legend">Tema</legend>
          <select
            className="select w-full"
            value={form.tema}
            onChange={(e) => setForm({ ...form, tema: e.target.value })}
          >
            <option value="light">Claro</option>
            <option value="dark">Escuro</option>
          </select>
        </fieldset>

        <div className="divider">Módulos ativos</div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          {([
            { key: "usar_turmas", label: "Turmas", desc: "Gerenciar turmas e vínculos" },
            { key: "usar_professores", label: "Professores", desc: "Cadastro e cronograma de professores" },
            { key: "usar_frequencia", label: "Frequência", desc: "Controle de presença" },
            { key: "usar_recuperacao", label: "Recuperação", desc: "Provas e notas de recuperação" },
          ] as const).map(({ key, label, desc }) => (
            <label key={key} className="cursor-pointer border border-base-300 rounded-lg p-3 flex items-start gap-3 hover:border-base-content/30 transition-colors">
              <input
                type="checkbox"
                className="checkbox mt-0.5"
                checked={!!form[key]}
                onChange={(e) => setForm({ ...form, [key]: e.target.checked })}
              />
              <div>
                <div className="font-medium text-sm">{label}</div>
                <div className="text-xs opacity-60">{desc}</div>
              </div>
            </label>
          ))}
        </div>

        <div className="pt-2 flex gap-3 flex-wrap">
          <button type="submit" className="btn btn-primary">Salvar Configurações</button>
        </div>
      </form>

      <div className="divider">Banco de Dados</div>
      <div className="flex gap-3 flex-wrap">
        <button type="button" className="btn btn-outline" onClick={handleBackup}>Exportar backup</button>
        <button type="button" className="btn btn-outline btn-warning" onClick={handleRestore}>Restaurar backup</button>
      </div>

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
