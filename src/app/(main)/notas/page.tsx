"use client";
import { useState, useEffect, useCallback } from "react";
import { MdAdd, MdEdit, MdDelete, MdPictureAsPdf, MdSettings } from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import { save } from "@tauri-apps/plugin-dialog";
import Toast from "@/components/Toast";
import ColorPicker from "@/components/ColorPicker";
import type { Nota, Aluno, Materia, Prova, CategoriaLancamento, ToastState } from "@/types";

interface NotaForm { aluno_id: string; prova_id: string; descricao: string; valor: string; categoria_id: string; [k: string]: unknown; }
interface CatForm { nome: string; cor: string; }

const EMPTY: NotaForm = { aluno_id: "", prova_id: "", descricao: "", valor: "", categoria_id: "" };
const EMPTY_CAT: CatForm = { nome: "", cor: "#6366f1" };

export default function NotasPage() {
  const [notas, setNotas] = useState<Nota[]>([]);
  const [alunos, setAlunos] = useState<Aluno[]>([]);
  const [materias, setMaterias] = useState<Materia[]>([]);
  const [provas, setProvas] = useState<Prova[]>([]);
  const [categorias, setCategorias] = useState<CategoriaLancamento[]>([]);
  const [form, setForm] = useState<NotaForm>(EMPTY);
  const [editing, setEditing] = useState<number | null>(null);
  const [modal, setModal] = useState(false);
  const [filterAluno, setFilterAluno] = useState("");
  const [toast, setToast] = useState<ToastState | null>(null);
  // Categoria CRUD modal
  const [catModal, setCatModal] = useState(false);
  const [catForm, setCatForm] = useState<CatForm>(EMPTY_CAT);
  const [catEditing, setCatEditing] = useState<number | null>(null);

  const load = useCallback(async () => {
    const [n, a, p, m, cats] = await Promise.all([
      invokeCmd<Nota[]>("list_notas"),
      invokeCmd<Aluno[]>("list_alunos"),
      invokeCmd<Prova[]>("list_provas"),
      invokeCmd<Materia[]>("list_materias"),
      invokeCmd<CategoriaLancamento[]>("list_categoria_lancamentos"),
    ]);
    setNotas(n);
    setAlunos(a);
    setProvas(p);
    setMaterias(m);
    setCategorias(cats);
  }, []);

  useEffect(() => { load(); }, [load]);

  function notify(message: string, type: ToastState["type"] = "success") {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  }

  const provasDoAluno = (() => {
    if (!form.aluno_id) return provas;
    const aluno = alunos.find((a) => String(a.id) === form.aluno_id);
    return provas.filter((p) => !aluno?.turma_id || !p.turma_id || p.turma_id === aluno.turma_id);
  })();

  function openCreate() {
    setEditing(null);
    setForm(EMPTY);
    setModal(true);
  }

  function openEdit(n: Nota) {
    setEditing(n.id);
    setForm({ aluno_id: String(n.aluno_id), prova_id: n.prova_id ? String(n.prova_id) : "", descricao: n.descricao, valor: String(n.valor), categoria_id: n.categoria_id ? String(n.categoria_id) : "" });
    setModal(true);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const valor = parseFloat(form.valor);
    if (!form.aluno_id) {
      notify("Selecione um aluno.", "error");
      return;
    }
    if (!form.valor || isNaN(valor)) {
      notify("Preencha o valor da nota.", "error");
      return;
    }
    if (valor < 0) {
      notify("A nota não pode ser negativa.", "error");
      return;
    }
    const maxNota = form.prova_id ? (provas.find((p) => String(p.id) === form.prova_id)?.valor_total ?? null) : null;
    if (maxNota !== null && valor > maxNota) {
      notify(`A nota não pode exceder o valor total da prova (${maxNota}).`, "error");
      return;
    }
    const payload = {
      alunoId: Number(form.aluno_id),
      provaId: form.prova_id ? Number(form.prova_id) : null,
      descricao: form.descricao,
      valor,
      categoriaId: form.categoria_id ? Number(form.categoria_id) : null,
    };
    try {
      if (editing !== null) {
        await invokeCmd("update_nota", { id: editing, ...payload });
        notify("Nota atualizada.");
      } else {
        await invokeCmd("create_nota", payload);
        notify("Nota lançada.");
      }
      setModal(false);
      load();
    } catch (err) {
      notify(String(err) || "Erro ao salvar.", "error");
    }
  }

  async function handleDelete(id: number) {
    try {
      await invokeCmd("delete_nota", { id });
      notify("Nota removida.");
      load();
    } catch {
      notify("Erro ao remover.", "error");
    }
  }

  // ── Categorias CRUD ─────────────────────────────────────────────
  function openCreateCat() { setCatEditing(null); setCatForm(EMPTY_CAT); setCatModal(true); }
  function openEditCat(c: CategoriaLancamento) { setCatEditing(c.id); setCatForm({ nome: c.nome, cor: c.cor }); }

  async function handleSaveCat(e: React.FormEvent) {
    e.preventDefault();
    if (!catForm.nome.trim()) return;
    try {
      if (catEditing !== null) {
        await invokeCmd("update_categoria_lancamento", { id: catEditing, nome: catForm.nome, cor: catForm.cor });
        notify("Categoria atualizada.");
      } else {
        await invokeCmd("create_categoria_lancamento", { nome: catForm.nome, cor: catForm.cor });
        notify("Categoria criada.");
      }
      setCatEditing(null);
      setCatForm(EMPTY_CAT);
      setCategorias(await invokeCmd<CategoriaLancamento[]>("list_categoria_lancamentos"));
    } catch (err) { notify(String(err), "error"); }
  }

  async function handleDeleteCat(id: number) {
    try {
      await invokeCmd("delete_categoria_lancamento", { id });
      notify("Categoria removida.");
      setCategorias(await invokeCmd<CategoriaLancamento[]>("list_categoria_lancamentos"));
    } catch (err) { notify(String(err), "error"); }
  }

  const boletim = (() => {
    if (!filterAluno) return null;
    const alunoNotas = notas.filter((n) => String(n.aluno_id) === filterAluno);
    const groups = new Map<number | null, typeof alunoNotas>();
    for (const n of alunoNotas) {
      const mid = provas.find((p) => p.id === n.prova_id)?.materia_id ?? null;
      if (!groups.has(mid)) groups.set(mid, []);
      groups.get(mid)!.push(n);
    }
    let totalPeso = 0, totalPond = 0;
    const grupos = [...groups.entries()].map(([mid, ns]) => {
      let peso = 0, pond = 0;
      for (const n of ns) { const w = provas.find((p) => p.id === n.prova_id)?.valor_total ?? 1; peso += w; pond += n.valor * w; }
      totalPeso += peso; totalPond += pond;
      return { nome: mid !== null ? (materias.find((m) => m.id === mid)?.nome ?? "?") : "Sem matéria", media: peso ? pond / peso : 0 };
    });
    return { grupos, mediaGeral: totalPeso ? totalPond / totalPeso : 0 };
  })();

  const filtered = filterAluno ? notas.filter((n) => String(n.aluno_id) === filterAluno) : notas;

  return (
    <div>
      <div className="flex items-center gap-3 mb-6 flex-wrap">
        <h1 className="text-3xl font-bold">Notas</h1>
        <div className="flex-1" />
        {filterAluno && (
          <button className="btn btn-outline gap-1" onClick={async () => {
            const aluno = alunos.find((a) => String(a.id) === filterAluno);
            const nome = (aluno?.nome ?? "aluno").replace(/[^\w\s]/g, "").trim().replace(/\s+/g, "_");
            const filePath = await save({ defaultPath: `boletim_${nome}.pdf`, filters: [{ name: "PDF", extensions: ["pdf"] }] });
            if (!filePath) return;
            try {
              await invokeCmd("export_boletim_pdf", { alunoId: Number(filterAluno), path: filePath });
              notify("Boletim exportado com sucesso.");
            } catch (e) { notify(`Erro ao exportar: ${e}`, "error"); }
          }}><MdPictureAsPdf size={18} /> Exportar Boletim</button>
        )}
        <button className="btn btn-primary" onClick={openCreate}><MdAdd size={20} /> Lançar Nota</button>
      </div>

      <div className="mb-4">
        <select className="select w-64" value={filterAluno} onChange={(e) => setFilterAluno(e.target.value)}>
          <option value="">Todos os alunos</option>
          {alunos.map((a) => <option key={a.id} value={a.id}>{a.nome}</option>)}
        </select>
      </div>

      <div className="overflow-x-auto">
        <table className="table table-zebra w-full">
          <thead>
            <tr>
              <th>Aluno</th>
              <th>Categoria</th>
              <th>Prova / Atividade</th>
              <th>Descrição</th>
              <th>Valor</th>
              <th>Atualizado</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {filtered.map((n) => (
              <tr key={n.id}>
                <td>{alunos.find((a) => a.id === n.aluno_id)?.nome ?? "-"}</td>
                <td>
                  {n.categoria_id ? (
                    <span className="flex items-center gap-1.5">
                      <span className="w-2.5 h-2.5 rounded-full shrink-0" style={{ backgroundColor: categorias.find(c => c.id === n.categoria_id)?.cor ?? "#6366f1" }} />
                      {n.categoria_nome}
                    </span>
                  ) : <span className="text-base-content/40">–</span>}
                </td>
                <td>{provas.find((p) => p.id === n.prova_id)?.titulo ?? "-"}</td>
                <td>{n.descricao}</td>
                <td>{n.valor}</td>
                <td>{n.updated_at ? new Date(n.updated_at).toLocaleString("pt-BR", { day: "2-digit", month: "2-digit", year: "numeric", hour: "2-digit", minute: "2-digit" }) : "-"}</td>
                <td className="flex gap-2">
                  <button className="btn btn-sm btn-ghost" onClick={() => openEdit(n)}><MdEdit /></button>
                  <button className="btn btn-sm btn-ghost text-error" onClick={() => handleDelete(n.id)}><MdDelete /></button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {boletim && (
        <div className="mt-8">
          <h2 className="text-xl font-bold mb-4">Boletim</h2>
          <div className="overflow-x-auto">
            <table className="table table-zebra w-full">
              <thead>
                <tr><th>Matéria</th><th>Média Ponderada</th></tr>
              </thead>
              <tbody>
                {boletim.grupos.map((g) => (
                  <tr key={g.nome}>
                    <td>{g.nome}</td>
                    <td>{g.media.toFixed(2)}</td>
                  </tr>
                ))}
              </tbody>
              <tfoot>
                <tr><td className="font-bold">Média Geral</td><td className="font-bold">{boletim.mediaGeral.toFixed(2)}</td></tr>
              </tfoot>
            </table>
          </div>
        </div>
      )}

      {modal && (
        <div className="modal modal-open">
          <div className="modal-box">
            <h3 className="font-bold text-lg mb-4">{editing ? "Editar" : "Lançar"} Nota</h3>
            <form onSubmit={handleSubmit} className="flex flex-col gap-4">
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Aluno</legend>
                <select className="select w-full" value={form.aluno_id} onChange={(e) => setForm({ ...form, aluno_id: e.target.value })} required>
                  <option value="">Selecione</option>
                  {alunos.map((a) => <option key={a.id} value={a.id}>{a.nome}</option>)}
                </select>
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend flex items-center justify-between">
                  <span>Categoria</span>
                  <button type="button" className="btn btn-xs btn-ghost gap-1" onClick={() => setCatModal(true)} title="Gerenciar categorias">
                    <MdSettings size={14} /> Gerenciar
                  </button>
                </legend>
                <select className="select w-full" value={form.categoria_id} onChange={(e) => setForm({ ...form, categoria_id: e.target.value })}>
                  <option value="">Sem categoria</option>
                  {categorias.map((c) => (
                    <option key={c.id} value={c.id}>{c.nome}</option>
                  ))}
                </select>
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Prova (opcional)</legend>
                <select className="select w-full" value={form.prova_id} onChange={(e) => setForm({ ...form, prova_id: e.target.value })}>
                  <option value="">Nenhuma</option>
                  {provasDoAluno.map((p) => <option key={p.id} value={p.id}>{p.titulo}</option>)}
                </select>
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Descrição</legend>
                <input className="input w-full" value={form.descricao} onChange={(e) => setForm({ ...form, descricao: e.target.value })} placeholder="Ex: Trabalho, Simulado..." />
              </fieldset>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Valor</legend>
                <input type="number" step="0.1" className="input w-full" value={form.valor} onChange={(e) => setForm({ ...form, valor: e.target.value })} placeholder="0.0" />
              </fieldset>
              <div className="modal-action">
                <button type="button" className="btn" onClick={() => setModal(false)}>Cancelar</button>
                <button type="submit" className="btn btn-primary">Salvar</button>
              </div>
            </form>
          </div>
        </div>
      )}

      {catModal && (
        <div className="modal modal-open" style={{ zIndex: 1100 }}>
          <div className="modal-box max-w-md">
            <h3 className="font-bold text-lg mb-4">Categorias de Lançamento</h3>

            {/* Form to create/edit */}
            <form onSubmit={handleSaveCat} className="flex flex-col gap-3 mb-4 p-3 bg-base-200 rounded-lg">
              <p className="text-sm font-medium">{catEditing !== null ? "Editando categoria" : "Nova categoria"}</p>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Nome</legend>
                <input className="input w-full" value={catForm.nome} onChange={(e) => setCatForm({ ...catForm, nome: e.target.value })} required placeholder="Ex: Trabalho" />
              </fieldset>
              <ColorPicker label="Cor" value={catForm.cor} onChange={(c) => setCatForm({ ...catForm, cor: c })} />
              <div className="flex gap-2">
                {catEditing !== null && (
                  <button type="button" className="btn btn-sm btn-ghost" onClick={() => { setCatEditing(null); setCatForm(EMPTY_CAT); }}>Cancelar</button>
                )}
                <button type="submit" className="btn btn-sm btn-primary ml-auto">{catEditing !== null ? "Atualizar" : "Adicionar"}</button>
              </div>
            </form>

            {/* List */}
            <ul className="space-y-2">
              {categorias.map((c) => (
                <li key={c.id} className="flex items-center gap-3 p-2 rounded-lg hover:bg-base-200">
                  <span className="w-4 h-4 rounded-full shrink-0" style={{ backgroundColor: c.cor }} />
                  <span className="flex-1 font-medium">{c.nome}</span>
                  <button className="btn btn-xs btn-ghost" onClick={() => openEditCat(c)}><MdEdit size={14} /></button>
                  <button className="btn btn-xs btn-ghost text-error" onClick={() => handleDeleteCat(c.id)}><MdDelete size={14} /></button>
                </li>
              ))}
              {categorias.length === 0 && <li className="text-sm text-base-content/40 text-center py-4">Nenhuma categoria.</li>}
            </ul>

            <div className="modal-action">
              <button className="btn" onClick={() => { setCatModal(false); setCatEditing(null); setCatForm(EMPTY_CAT); }}>Fechar</button>
            </div>
          </div>
        </div>
      )}

      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
    </div>
  );
}
