"use client";
import { useEditor, EditorContent } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import Mathematics from "@tiptap/extension-mathematics";
import Image from "@tiptap/extension-image";
import { Table } from "@tiptap/extension-table";
import TableRow from "@tiptap/extension-table-row";
import TableHeader from "@tiptap/extension-table-header";
import TableCell from "@tiptap/extension-table-cell";
import TextAlign from "@tiptap/extension-text-align";
import Underline from "@tiptap/extension-underline";
import { TextStyle } from "@tiptap/extension-text-style";
import "katex/dist/katex.min.css";
import { useState, useCallback, useEffect, useRef } from "react";
import {
  MdFormatBold, MdFormatItalic, MdFormatUnderlined, MdFormatListBulleted,
  MdFormatListNumbered, MdFormatAlignLeft, MdFormatAlignCenter, MdFormatAlignRight,
  MdTableChart, MdImage, MdFunctions, MdFormatQuote, MdOutlineHorizontalRule,
  MdUndo, MdRedo,
} from "react-icons/md";
import { open } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";

// ── Math modal ───────────────────────────────────────────────────────────────

const MATH_SNIPPETS = [
  { label: "Fração", latex: "\\frac{a}{b}" },
  { label: "Potência", latex: "x^{n}" },
  { label: "Raiz quadrada", latex: "\\sqrt{x}" },
  { label: "Raiz n-ésima", latex: "\\sqrt[n]{x}" },
  { label: "Somatório", latex: "\\sum_{i=1}^{n} a_i" },
  { label: "Integral", latex: "\\int_{a}^{b} f(x)\\,dx" },
  { label: "Limite", latex: "\\lim_{x \\to \\infty} f(x)" },
  { label: "Pi (π)", latex: "\\pi" },
  { label: "Infinito (∞)", latex: "\\infty" },
  { label: "Delta (Δ)", latex: "\\Delta" },
  { label: "Seta direita", latex: "\\rightarrow" },
  { label: "Seta dupla", latex: "\\Leftrightarrow" },
  { label: "Diferente (≠)", latex: "\\neq" },
  { label: "Menor ou igual (≤)", latex: "\\leq" },
  { label: "Maior ou igual (≥)", latex: "\\geq" },
  { label: "Pertence (∈)", latex: "\\in" },
  { label: "Não pertence (∉)", latex: "\\notin" },
  { label: "Ângulo", latex: "\\angle ABC" },
  { label: "Grau (°)", latex: "90^{\\circ}" },
  { label: "Sistema linear", latex: "\\begin{cases} ax + by = c \\\\ dx + ey = f \\end{cases}" },
  { label: "Logaritmo", latex: "\\log_{b}(x)" },
  { label: "Log natural", latex: "\\ln(x)" },
  { label: "Modulo / valor absoluto", latex: "|x|" },
  { label: "Vetor", latex: "\\vec{v}" },
  { label: "Média aritmética", latex: "\\bar{x} = \\frac{\\sum x_i}{n}" },
];

interface MathModalProps {
  onInsert: (latex: string) => void;
  onClose: () => void;
}
function MathModal({ onInsert, onClose }: MathModalProps) {
  const [input, setInput] = useState("");
  const [preview, setPreview] = useState<string | null>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
    // lazy import katex only in browser
    if (!input.trim()) { setPreview(null); return; }
    import("katex").then((katex) => {
      try {
        setPreview(katex.default.renderToString(input, { displayMode: true, throwOnError: false }));
      } catch {
        setPreview(null);
      }
    });
  }, [input]);

  function handleSnippet(latex: string) {
    setInput((prev) => prev + (prev && !prev.endsWith(" ") ? " " : "") + latex + " ");
    inputRef.current?.focus();
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4" onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}>
      <div className="bg-base-100 rounded-box shadow-2xl w-full max-w-2xl flex flex-col max-h-[90vh]">
        <div className="flex items-center justify-between p-4 border-b border-base-300">
          <div>
            <h3 className="text-lg font-bold">Inserir Fórmula Matemática</h3>
            <p className="text-xs text-base-content/50 mt-0.5">Digite a fórmula em LaTeX ou clique em um modelo abaixo</p>
          </div>
          <button className="btn btn-sm btn-ghost" onClick={onClose}>✕</button>
        </div>

        {/* Snippets */}
        <div className="p-4 border-b border-base-300 overflow-y-auto max-h-40">
          <p className="text-xs font-semibold text-base-content/50 mb-2 uppercase tracking-wide">Modelos prontos</p>
          <div className="flex flex-wrap gap-1.5">
            {MATH_SNIPPETS.map((s) => (
              <button key={s.label} className="btn btn-xs btn-outline" onClick={() => handleSnippet(s.latex)}>
                {s.label}
              </button>
            ))}
          </div>
        </div>

        {/* Input */}
        <div className="p-4 flex flex-col gap-3 flex-1 overflow-auto">
          <div>
            <label className="text-sm font-medium mb-1 block">Código LaTeX</label>
            <textarea
              ref={inputRef}
              className="textarea w-full font-mono text-sm"
              rows={3}
              placeholder="Ex: \frac{x^2 + 1}{2}"
              value={input}
              onChange={(e) => setInput(e.target.value)}
            />
          </div>

          {/* Preview */}
          <div className="min-h-16 flex items-center justify-center bg-base-200 rounded-box p-4">
            {preview
              ? <div dangerouslySetInnerHTML={{ __html: preview }} className="text-base-content" />
              : <span className="text-base-content/30 text-sm italic">A pré-visualização aparece aqui</span>
            }
          </div>
        </div>

        <div className="flex justify-end gap-2 p-4 border-t border-base-300">
          <button className="btn btn-ghost" onClick={onClose}>Cancelar</button>
          <button
            className="btn btn-primary"
            disabled={!input.trim()}
            onClick={() => { if (input.trim()) { onInsert(input.trim()); onClose(); } }}
          >
            Inserir Fórmula
          </button>
        </div>
      </div>
    </div>
  );
}

// ── Toolbar button ────────────────────────────────────────────────────────────

function ToolBtn({
  onClick, active, disabled, title, children,
}: {
  onClick: () => void; active?: boolean; disabled?: boolean; title: string; children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onMouseDown={(e) => { e.preventDefault(); onClick(); }}
      disabled={disabled}
      title={title}
      className={`btn btn-xs ${active ? "btn-primary" : "btn-ghost"} px-1.5`}
    >
      {children}
    </button>
  );
}

// ── Separator ─────────────────────────────────────────────────────────────────

function Sep() { return <span className="w-px h-5 bg-base-300 mx-0.5 self-center" />; }

// ── Main component ────────────────────────────────────────────────────────────

interface Props {
  value: string;
  onChange: (html: string) => void;
  placeholder?: string;
  minHeight?: number;
  label?: string;
}

export default function RichEditor({ value, onChange, placeholder, minHeight = 100, label }: Props) {
  const [mathOpen, setMathOpen] = useState(false);

  const editor = useEditor({
    immediatelyRender: false,
    extensions: [
      StarterKit,
      Underline,
      TextStyle,
      TextAlign.configure({ types: ["heading", "paragraph"] }),
      Mathematics,
      Image.configure({ allowBase64: true }),
      Table.configure({ resizable: false }),
      TableRow,
      TableHeader,
      TableCell,
    ],
    content: value || "",
    editorProps: {
      attributes: {
        class: "prose prose-sm max-w-none focus:outline-none p-3 min-h-[var(--editor-min-h)]",
        style: `--editor-min-h: ${minHeight}px`,
      },
    },
    onUpdate: ({ editor }) => onChange(editor.getHTML()),
  });

  // Sync external value changes (e.g. load from DB) only when truly different
  useEffect(() => {
    if (!editor) return;
    if (editor.getHTML() !== value) {
      editor.commands.setContent(value || "");
    }
  }, [value, editor]);

  const insertMath = useCallback((latex: string) => {
    // Use insertInlineMath command so the content is stored as a proper Tiptap node:
    // <span data-type="inline-math" data-latex="..."> — which the Rust export pipeline reads correctly.
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (editor?.chain().focus() as any).insertInlineMath({ latex }).run();
  }, [editor]);

  async function insertImageFromFile() {
    const filePath = await open({
      filters: [{ name: "Imagem", extensions: ["png", "jpg", "jpeg", "gif", "webp", "bmp"] }],
      multiple: false,
    });
    if (!filePath || typeof filePath !== "string") return;
    const bytes = await readFile(filePath);
    const ext = filePath.split(".").pop()?.toLowerCase() ?? "png";
    const mime = ext === "jpg" || ext === "jpeg" ? "image/jpeg" : `image/${ext}`;
    const b64 = btoa(String.fromCharCode(...bytes));
    editor?.chain().focus().setImage({ src: `data:${mime};base64,${b64}` }).run();
  }

  if (!editor) return null;

  return (
    <>
      {mathOpen && (
        <MathModal
          onInsert={insertMath}
          onClose={() => setMathOpen(false)}
        />
      )}

      <fieldset className="fieldset">
        {label && <legend className="fieldset-legend">{label}</legend>}
        <div className="border border-base-300 rounded-box overflow-hidden bg-base-100 focus-within:border-primary transition-colors">
        {/* Toolbar */}
        <div className="flex flex-wrap items-center gap-0.5 p-1.5 bg-base-200 border-b border-base-300">
          {/* Undo/Redo */}
          <ToolBtn onClick={() => editor.chain().focus().undo().run()} disabled={!editor.can().undo()} title="Desfazer (Ctrl+Z)">
            <MdUndo size={15} />
          </ToolBtn>
          <ToolBtn onClick={() => editor.chain().focus().redo().run()} disabled={!editor.can().redo()} title="Refazer (Ctrl+Y)">
            <MdRedo size={15} />
          </ToolBtn>
          <Sep />

          {/* Formatting */}
          <ToolBtn onClick={() => editor.chain().focus().toggleBold().run()} active={editor.isActive("bold")} title="Negrito (Ctrl+B)">
            <MdFormatBold size={15} />
          </ToolBtn>
          <ToolBtn onClick={() => editor.chain().focus().toggleItalic().run()} active={editor.isActive("italic")} title="Itálico (Ctrl+I)">
            <MdFormatItalic size={15} />
          </ToolBtn>
          <ToolBtn onClick={() => editor.chain().focus().toggleUnderline().run()} active={editor.isActive("underline")} title="Sublinhado (Ctrl+U)">
            <MdFormatUnderlined size={15} />
          </ToolBtn>
          <Sep />

          {/* Headings */}
          <ToolBtn onClick={() => editor.chain().focus().toggleHeading({ level: 2 }).run()} active={editor.isActive("heading", { level: 2 })} title="Título">
            <span className="text-xs font-bold">H2</span>
          </ToolBtn>
          <ToolBtn onClick={() => editor.chain().focus().toggleHeading({ level: 3 }).run()} active={editor.isActive("heading", { level: 3 })} title="Subtítulo">
            <span className="text-xs font-bold">H3</span>
          </ToolBtn>
          <Sep />

          {/* Alignment */}
          <ToolBtn onClick={() => editor.chain().focus().setTextAlign("left").run()} active={editor.isActive({ textAlign: "left" })} title="Alinhar à esquerda">
            <MdFormatAlignLeft size={15} />
          </ToolBtn>
          <ToolBtn onClick={() => editor.chain().focus().setTextAlign("center").run()} active={editor.isActive({ textAlign: "center" })} title="Centralizar">
            <MdFormatAlignCenter size={15} />
          </ToolBtn>
          <ToolBtn onClick={() => editor.chain().focus().setTextAlign("right").run()} active={editor.isActive({ textAlign: "right" })} title="Alinhar à direita">
            <MdFormatAlignRight size={15} />
          </ToolBtn>
          <Sep />

          {/* Lists */}
          <ToolBtn onClick={() => editor.chain().focus().toggleBulletList().run()} active={editor.isActive("bulletList")} title="Lista com marcadores">
            <MdFormatListBulleted size={15} />
          </ToolBtn>
          <ToolBtn onClick={() => editor.chain().focus().toggleOrderedList().run()} active={editor.isActive("orderedList")} title="Lista numerada">
            <MdFormatListNumbered size={15} />
          </ToolBtn>
          <ToolBtn onClick={() => editor.chain().focus().toggleBlockquote().run()} active={editor.isActive("blockquote")} title="Citação">
            <MdFormatQuote size={15} />
          </ToolBtn>
          <ToolBtn onClick={() => editor.chain().focus().setHorizontalRule().run()} title="Linha horizontal">
            <MdOutlineHorizontalRule size={15} />
          </ToolBtn>
          <Sep />

          {/* Table */}
          <ToolBtn
            onClick={() => editor.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run()}
            title="Inserir tabela"
          >
            <MdTableChart size={15} />
          </ToolBtn>
          <Sep />

          {/* Image */}
          <ToolBtn onClick={insertImageFromFile} title="Inserir imagem">
            <MdImage size={15} />
          </ToolBtn>
          <Sep />

          {/* Math — destaque especial */}
          <button
            type="button"
            onMouseDown={(e) => { e.preventDefault(); setMathOpen(true); }}
            title="Inserir fórmula matemática"
            className="btn btn-xs btn-secondary gap-1 px-2"
          >
            <MdFunctions size={15} />
            <span className="hidden sm:inline text-xs">Fórmula</span>
          </button>
        </div>

        {/* Editor area */}
        <div className="relative">
          {editor.isEmpty && placeholder && (
            <p className="absolute top-3 left-3 text-base-content/30 pointer-events-none text-sm select-none">
              {placeholder}
            </p>
          )}
          <EditorContent editor={editor} />
        </div>

        {/* Table context menu */}
        {editor.isActive("table") && (
          <div className="flex flex-wrap gap-1 p-1.5 bg-base-200 border-t border-base-300 text-xs">
            <span className="text-base-content/40 self-center mr-1">Tabela:</span>
            <button type="button" className="btn btn-xs btn-ghost" onMouseDown={(e) => { e.preventDefault(); editor.chain().focus().addColumnAfter().run(); }}>+ Coluna</button>
            <button type="button" className="btn btn-xs btn-ghost" onMouseDown={(e) => { e.preventDefault(); editor.chain().focus().addRowAfter().run(); }}>+ Linha</button>
            <button type="button" className="btn btn-xs btn-ghost" onMouseDown={(e) => { e.preventDefault(); editor.chain().focus().deleteColumn().run(); }}>− Coluna</button>
            <button type="button" className="btn btn-xs btn-ghost" onMouseDown={(e) => { e.preventDefault(); editor.chain().focus().deleteRow().run(); }}>− Linha</button>
            <button type="button" className="btn btn-xs btn-ghost text-error" onMouseDown={(e) => { e.preventDefault(); editor.chain().focus().deleteTable().run(); }}>Excluir tabela</button>
          </div>
        )}
        </div>
      </fieldset>
    </>
  );
}
