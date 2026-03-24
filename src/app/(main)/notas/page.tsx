"use client";
import { useState, useEffect, useCallback } from "react";
import { MdAdd, MdEdit, MdDelete, MdPictureAsPdf, MdSettings } from "react-icons/md";
import { invokeCmd } from "@/utils/tauri";
import { save } from "@tauri-apps/plugin-dialog";
import Toast from "@/components/Toast";
import Modal from "@/components/Modal";
import ColorPicker from "@/components/ColorPicker";
import type { Nota, Aluno, Materia, Prova, Turma, CategoriaLancamento, Configuracoes, ToastState } from "@/types";

interface NotaForm {
  turma_id: string;
  aluno_id: string;
  categoria_id: string;
  prova_materia_id: string;
  prova_bimestre: string;
  prova_ano: string;
  prova_id: string;
  valor: string;
}
interface CatForm { nome: string; cor: string; vincula_provas: boolean; }

const EMPTY: NotaForm = { turma_id: "", aluno_id: "", categoria_id: "", prova_materia_id: "", prova_bimestre: "", prova_ano: "", prova_id: "", valor: "" };
const EMPTY_CAT: CatForm = { nome: "", cor: "#6366f1", vincula_provas: false };
const BIMESTRES = [1, 2, 3, 4];

export default function NotasPage() {
  const [notas, setNotas] = useState<Nota[]>([]);
  const [alunos, setAlunos] = useState<Aluno[]>([]);
  const [turmas, setTurmas] = useState<Turma[]>([]);
  const [materias, setMaterias] = useState<Materia[]>([]);
  const [provas, setProvas] = useState<Prova[]>([]);
  const [categorias, setCategorias] = useState<CategoriaLancamento[]>([]);
  const [config, setConfig] = useState<Configuracoes | null>(null);
  const [toast, setToast] = useState<ToastState | null>(null);

  const [filterBimestre, setFilterBimestre] = useState("");
  const [filterAno, setFilterAno] = useState("");
  const [filterTurmaId, setFilterTurmaId] = useState("");
  const [filterMateriaId, setFilterMateriaId] = useState("");
  const [filterAlunoId, setFilterAlunoId] = useState("");

  const [form, setForm] = useState<NotaForm>(EMPTY);
  const [editing, setEditing] = useState<number | null>(null);
  const [modal, setModal] = useState(false);
  const [deleteId, setDeleteId] = useState<number | null>(null);

  const [catModal, setCatModal] = useState(false);
  const [catForm, setCatForm] = useState<CatForm>(EMPTY_CAT);
  const [catEditing, setCatEditing] = useState<number | null>(null);
  const [deleteCatId, setDeleteCatId] = useState<number | null>(null);

  const loadNotas = useCallback(async () => {
    const n = await invokeCmd<Nota[]>("list_notas", {
      bimestre: filterBimestre ? Number(filterBimestre) : null,
      ano: filterAno || null,
      turmaId: filterTurmaId ? Number(filterTurmaId) : null,
      materiaId: filterMateriaId ? Number(filterMateriaId) : null,
      alunoId: filterAlunoId ? Number(filterAlunoId) : null,
    });
    setNotas(n);
  }, [filterBimestre, filterAno, filterTurmaId, filterMateriaId, filterAlunoId]);

  const load = useCallback(async () => {
    const [a, t, m, p, cats, cfg] = await Promise.all([
      invokeCmd<Aluno[]>("list_alunos"),
      invokeCmd<Turma[]>("list_turmas"),
      invokeCmd<Materia[]>("list_materias"),
      invokeCmd<Prova[]>("list_provas"),
      invokeCmd<CategoriaLancamento[]>("list_categoria_lancamentos"),
      invokeCmd<Configuracoes>("get_configuracoes"),
    ]);
    setAlunos(a); setTurmas(t); setMaterias(m); setProvas(p); setCategorias(cats); setConfig(cfg);
  }, []);

  useEffect(() => { load(); }, [load]);
  useEffect(() => { loadNotas(); }, [loadNotas]);

  function notify(message: string, type: ToastState["type"] = "success") {
    setToast({ message, type });
    setTimeout(() => setToast(null), 3000);
  }

  const catSelecionada = categorias.find(c => String(c.id) === form.categoria_id);
  const isProvaCat = catSelecionada?.vincula_provas ?? false;

  const alunosFiltrados = config?.usar_turmas && form.turma_id
    ? alunos.filter(a => a.turma_id === Number(form.turma_id))
    : alunos;

  const anosDisponiveis = [...new Set(provas.map(p => p.ano_letivo).filter(Boolean))].sort();

  const provasFiltradas = provas.filter(p => {
    if (isProvaCat) {
      if (form.prova_materia_id && p.materia_id !== Number(form.prova_materia_id)) return false;
      if (form.prova_bimestre && p.bimestre !== Number(form.prova_bimestre)) return false;
      if (form.prova_ano && p.ano_letivo !== form.prova_ano) return false;
    }
    if (config?.usar_turmas && form.turma_id && p.turma_id && p.turma_id !== Number(form.turma_id)) return false;
    return true;
  });

  function openCreate() {
    setEditing(null);
    setForm(EMPTY);
    setModal(true);
  }

  function openEdit(n: Nota) {
    setEditing(n.id);
    const prova = provas.find(p => p.id === n.prova_id);
    const aluno = alunos.find(a => a.id === n.aluno_id);
    setForm({
      turma_id: aluno?.turma_id ? String(aluno.turma_id) : "",
      aluno_id: String(n.aluno_id),
      categoria_id: n.categoria_id ? String(n.categoria_id) : "",
      prova_materia_id: prova?.materia_id ? String(prova.materia_id) : "",
      prova_bimestre: prova ? String(prova.bimestre) : "",
      prova_ano: prova?.ano_letivo ?? "",
      prova_id: n.prova_id ? String(n.prova_id) : "",
      valor: String(n.valor),
    });
    setModal(true);
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    const valor = parseFloat(form.valor);
    if (!form.aluno_id) { notify("Selecione um aluno.", "error"); return; }
    if (!form.valor || isNaN(valor)) { notify("Preencha o valor da nota.", "error"); return; }
    if (valor < 0) { notify("A nota não pode ser negativa.", "error"); return; }
    if (isProvaCat && form.prova_id) {
      const maxNota = provas.find(p => String(p.id) === form.prova_id)?.valor_total ?? null;
      if (maxNota !== null && valor > maxNota) {
        notify(`A nota não pode exceder o valor total da prova (${maxNota}).`, "error");
        return;
      }
    }
    const payload = {
      alunoId: Number(form.aluno_id),
      provaId: isProvaCat && form.prova_id ? Number(form.prova_id) : null,
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
      loadNotas();
    } catch (err) {
      notify(String(err) || "Erro ao salvar.", "error");
    }
  }

  async function handleDelete(id: number) {
    try {
      await invokeCmd("delete_nota", { id });
      notify("Nota removida.");
      setDeleteId(null);
      loadNotas();
    } catch {
      notify("Erro ao remover.", "error");
    }
  }

  function openEditCat(c: CategoriaLancamento) {
    setCatEditing(c.id);
    setCatForm({ nome: c.nome, cor: c.cor, vincula_provas: c.vincula_provas });
  }

  async function handleSaveCat(e: React.FormEvent) {
    e.preventDefault();
    if (!catForm.nome.trim()) return;
    try {
      if (catEditing !== null) {
        await invokeCmd("update_categoria_lancamento", { id: catEditing, nome: catForm.nome, cor: catForm.cor, vinculaProvas: catForm.vincula_provas });
        notify("Categoria atualizada.");
      } else {
        await invokeCmd("create_categoria_lancamento", { nome: catForm.nome, cor: catForm.cor, vinculaProvas: catForm.vincula_provas });
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
      setDeleteCatId(null);
      setCategorias(await invokeCmd<CategoriaLancamento[]>("list_categoria_lancamentos"));
    } catch (err) { notify(String(err), "error"); }
  }

  const boletim = (() => {
    if (!filterAlunoId) return null;
    const alunoNotas = notas.filter(n => String(n.aluno_id) === filterAlunoId);
    const groups = new Map<number | null, typeof alunoNotas>();
    for (const n of alunoNotas) {
      const mid = provas.find(p => p.id === n.prova_id)?.materia_id ?? null;
      if (!groups.has(mid)) groups.set(mid, []);
      groups.get(mid)!.push(n);
    }
    let totalPeso = 0, totalPond = 0;
    const grupos = [...groups.entries()].map(([mid, ns]) => {
      let peso = 0, pond = 0;
      for (const n of ns) { const w = provas.find(p => p.id === n.prova_id)?.valor_total ?? 1; peso += w; pond += n.valor * w; }
      totalPeso += peso; totalPond += pond;
      return { nome: mid !== null ? (materias.find(m => m.id === mid)?.nome ?? "?") : "Sem matéria", media: peso ? pond / peso : 0 };
    });
    return { grupos, mediaGeral: totalPeso ? totalPond / totalPeso : 0 };
  })();

  return (
    <div>
      <div className="flex items-center gap-3 mb-6 flex-wrap">
        <h1 className="text-3xl font-bold">Notas</h1>
        <div className="flex-1" />
        {filterAlunoId && (
          <button className="btn btn-outline gap-1" onClick={async () => {
            const aluno = alunos.find(a => String(a.id) === filterAlunoId);
            const nome = (aluno?.nome ?? "aluno").replace(/[^\w\s]/g, "").trim().replace(/\s+/g, "_");
            const filePath = await save({ defaultPath: `boletim_${nome}.pdf`, filters: [{ name: "PDF", extensions: ["pdf"] }] });
            if (!filePath) return;
            try {
              await invokeCmd("export_boletim_pdf", { alunoId: Number(filterAlunoId), path: filePath });
              notify("Boletim exportado com sucesso.");
            } catch (e) { notify(`Erro ao exportar: ${e}`, "error"); }
          }}><MdPictureAsPdf size={18} /> Exportar Boletim</button>
        )}
        <button className="btn btn-primary" onClick={openCreate}><MdAdd size={20} /> Lançar Nota</button>
      </div>

      <div className="flex flex-wrap gap-3 mb-6">
        <fieldset className="fieldset">
          <legend className="fieldset-legend">Bimestre</legend>
          <select className="select select-sm" value={filterBimestre} onChange={e => setFilterBimestre(e.target.value)}>
            <option value="">Todos</option>
            {BIMESTRES.map(b => <option key={b} value={b}>{b}º</option>)}
          </select>
        </fieldset>
        <fieldset className="fieldset">
          <legend className="fieldset-legend">Ano letivo</legend>
          <select className="select select-sm" value={filterAno} onChange={e => setFilterAno(e.target.value)}>
            <option value="">Todos</option>
            {anosDisponiveis.map(a => <option key={a} value={a}>{a}</option>)}
          </select>
        </fieldset>
        {config?.usar_turmas && (
          <fieldset className="fieldset">
            <legend className="fieldset-legend">Turma</legend>
            <select className="select select-sm" value={filterTurmaId} onChange={e => setFilterTurmaId(e.target.value)}>
              <option value="">Todas</option>
              {turmas.map(t => <option key={t.id} value={t.id}>{t.nome}</option>)}
            </select>
          </fieldset>
        )}
        <fieldset className="fieldset">
          <legend className="fieldset-legend">Matéria</legend>
          <select className="select select-sm" value={filterMateriaId} onChange={e => setFilterMateriaId(e.target.value)}>
            <option value="">Todas</option>
            {materias.map(m => <option key={m.id} value={m.id}>{m.nome}</option>)}
          </select>
        </fieldset>
        <fieldset className="fieldset">
          <legend className="fieldset-legend">Aluno</legend>
          <select className="select select-sm" value={filterAlunoId} onChange={e => setFilterAlunoId(e.target.value)}>
            <option value="">Todos</option>
            {alunos.map(a => <option key={a.id} value={a.id}>{a.nome}</option>)}
          </select>
        </fieldset>
      </div>

      <div className="overflow-x-auto">
        <table className="table table-zebra w-full">
          <thead>
            <tr>
              <th>Aluno</th>
              <th>Categoria</th>
              <th>Prova</th>
              <th>Valor</th>
              <th>Atualizado</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {notas.map(n => (
              <tr key={n.id}>
                <td>{alunos.find(a => a.id === n.aluno_id)?.nome ?? "-"}</td>
                <td>
                  {n.categoria_id ? (
                    <span className="flex items-center gap-1.5">
                      <span className="w-2.5 h-2.5 rounded-full shrink-0" style={{ backgroundColor: categorias.find(c => c.id === n.categoria_id)?.cor ?? "#6366f1" }} />
                      {n.categoria_nome}
                    </span>
                  ) : <span className="text-base-content/40">–</span>}
                </td>
                <td>{provas.find(p => p.id === n.prova_id)?.titulo ?? "-"}</td>
                <td>{n.valor}</td>
                <td>{n.updated_at ? new Date(n.updated_at).toLocaleString("pt-BR", { day: "2-digit", month: "2-digit", year: "numeric", hour: "2-digit", minute: "2-digit" }) : "-"}</td>
                <td className="flex gap-2">
                  <button className="btn btn-sm btn-ghost" onClick={() => openEdit(n)}><MdEdit /></button>
                  <button className="btn btn-sm btn-ghost text-error" onClick={() => setDeleteId(n.id)}><MdDelete /></button>
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
              <thead><tr><th>Matéria</th><th>Média Ponderada</th></tr></thead>
              <tbody>
                {boletim.grupos.map(g => (
                  <tr key={g.nome}><td>{g.nome}</td><td>{g.media.toFixed(2)}</td></tr>
                ))}
              </tbody>
              <tfoot>
                <tr><td className="font-bold">Média Geral</td><td className="font-bold">{boletim.mediaGeral.toFixed(2)}</td></tr>
              </tfoot>
            </table>
          </div>
        </div>
      )}

      <Modal open={modal} onClose={() => setModal(false)} title={editing !== null ? "Editar Nota" : "Lançar Nota"} size="lg">
        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          {config?.usar_turmas && (
            <fieldset className="fieldset">
              <legend className="fieldset-legend">Turma</legend>
              <select className="select w-full" value={form.turma_id} onChange={e => setForm({ ...form, turma_id: e.target.value, aluno_id: "" })}>
                <option value="">Selecione a turma</option>
                {turmas.map(t => <option key={t.id} value={t.id}>{t.nome}</option>)}
              </select>
            </fieldset>
          )}
          <fieldset className="fieldset">
            <legend className="fieldset-legend">Aluno</legend>
            <select className="select w-full" value={form.aluno_id} onChange={e => setForm({ ...form, aluno_id: e.target.value })} required>
              <option value="">Selecione</option>
              {alunosFiltrados.map(a => <option key={a.id} value={a.id}>{a.nome}</option>)}
            </select>
          </fieldset>
          <fieldset className="fieldset">
            <legend className="fieldset-legend flex items-center justify-between">
              <span>Categoria</span>
              <button type="button" className="btn btn-xs btn-ghost gap-1" onClick={() => setCatModal(true)}><MdSettings size={14} /> Gerenciar</button>
            </legend>
            <select className="select w-full" value={form.categoria_id} onChange={e => setForm({ ...form, categoria_id: e.target.value, prova_id: "", prova_materia_id: "", prova_bimestre: "", prova_ano: "" })}>
              <option value="">Sem categoria</option>
              {categorias.map(c => <option key={c.id} value={c.id}>{c.nome}</option>)}
            </select>
          </fieldset>
          {isProvaCat && (
            <>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Matéria</legend>
                <select className="select w-full" value={form.prova_materia_id} onChange={e => setForm({ ...form, prova_materia_id: e.target.value, prova_id: "" })}>
                  <option value="">Todas</option>
                  {materias.map(m => <option key={m.id} value={m.id}>{m.nome}</option>)}
                </select>
              </fieldset>
              <div className="grid grid-cols-2 gap-3">
                <fieldset className="fieldset">
                  <legend className="fieldset-legend">Bimestre</legend>
                  <select className="select w-full" value={form.prova_bimestre} onChange={e => setForm({ ...form, prova_bimestre: e.target.value, prova_id: "" })}>
                    <option value="">Todos</option>
                    {BIMESTRES.map(b => <option key={b} value={b}>{b}º</option>)}
                  </select>
                </fieldset>
                <fieldset className="fieldset">
                  <legend className="fieldset-legend">Ano</legend>
                  <select className="select w-full" value={form.prova_ano} onChange={e => setForm({ ...form, prova_ano: e.target.value, prova_id: "" })}>
                    <option value="">Todos</option>
                    {anosDisponiveis.map(a => <option key={a} value={a}>{a}</option>)}
                  </select>
                </fieldset>
              </div>
              <fieldset className="fieldset">
                <legend className="fieldset-legend">Prova</legend>
                <select className="select w-full" value={form.prova_id} onChange={e => setForm({ ...form, prova_id: e.target.value })}>
                  <option value="">Nenhuma</option>
                  {provasFiltradas.map(p => <option key={p.id} value={p.id}>{p.titulo}</option>)}
                </select>
              </fieldset>
            </>
          )}
          <fieldset className="fieldset">
            <legend className="fieldset-legend">Valor</legend>
            <input type="number" step="0.1" min="0" className="input w-full" value={form.valor} onChange={e => setForm({ ...form, valor: e.target.value })} placeholder="0.0" required />
          </fieldset>
          <div className="modal-action">
            <button type="button" className="btn" onClick={() => setModal(false)}>Cancelar</button>
            <button type="submit" className="btn btn-primary">Salvar</button>
          </div>
        </form>
      </Modal>

      <Modal open={deleteId !== null} onClose={() => setDeleteId(null)} title="Excluir Nota" variant="confirm" color="error" confirmLabel="Excluir" onConfirm={() => deleteId !== null && handleDelete(deleteId)}>
        Deseja realmente excluir esta nota? Esta ação não pode ser desfeita.
      </Modal>

      <Modal open={catModal} onClose={() => { setCatModal(false); setCatEditing(null); setCatForm(EMPTY_CAT); }} title="Categorias de Lançamento" size="md">
        <form onSubmit={handleSaveCat} className="flex flex-col gap-3 mb-4 p-3 bg-base-200 rounded-lg">
          <p className="text-sm font-medium">{catEditing !== null ? "Editando categoria" : "Nova categoria"}</p>
          <fieldset className="fieldset">
            <legend className="fieldset-legend">Nome</legend>
            <input className="input w-full" value={catForm.nome} onChange={e => setCatForm({ ...catForm, nome: e.target.value })} required placeholder="Ex: Trabalho" />
          </fieldset>
          <ColorPicker label="Cor" value={catForm.cor} onChange={c => setCatForm({ ...catForm, cor: c })} />
          <label className="flex items-center gap-2 cursor-pointer">
            <input type="checkbox" className="checkbox checkbox-sm" checked={catForm.vincula_provas} onChange={e => setCatForm({ ...catForm, vincula_provas: e.target.checked })} />
            <span className="text-sm">Vincula ao sistema de provas</span>
          </label>
          <div className="flex gap-2">
            {catEditing !== null && (
              <button type="button" className="btn btn-sm btn-ghost" onClick={() => { setCatEditing(null); setCatForm(EMPTY_CAT); }}>Cancelar</button>
            )}
            <button type="submit" className="btn btn-sm btn-primary ml-auto">{catEditing !== null ? "Atualizar" : "Adicionar"}</button>
          </div>
        </form>
        <ul className="space-y-2">
          {categorias.map(c => (
            <li key={c.id} className="flex items-center gap-3 p-2 rounded-lg hover:bg-base-200">
              <span className="w-4 h-4 rounded-full shrink-0" style={{ backgroundColor: c.cor }} />
              <span className="flex-1 font-medium">{c.nome}</span>
              {c.vincula_provas && <span className="badge badge-sm badge-outline">Provas</span>}
              <button className="btn btn-xs btn-ghost" type="button" onClick={() => openEditCat(c)}><MdEdit size={14} /></button>
              <button className="btn btn-xs btn-ghost text-error" type="button" onClick={() => setDeleteCatId(c.id)}><MdDelete size={14} /></button>
            </li>
          ))}
          {categorias.length === 0 && <li className="text-sm text-base-content/40 text-center py-4">Nenhuma categoria.</li>}
        </ul>
        <div className="modal-action">
          <button className="btn" type="button" onClick={() => { setCatModal(false); setCatEditing(null); setCatForm(EMPTY_CAT); }}>Fechar</button>
        </div>
      </Modal>

      <Modal open={deleteCatId !== null} onClose={() => setDeleteCatId(null)} title="Excluir Categoria" variant="confirm" color="error" confirmLabel="Excluir" onConfirm={() => deleteCatId !== null && handleDeleteCat(deleteCatId)}>
        Deseja realmente excluir esta categoria? Esta ação não pode ser desfeita.
      </Modal>

      {toast && <Toast message={toast.message} type={toast.type} onClose={() => setToast(null)} />}
    </div>
  );
}
