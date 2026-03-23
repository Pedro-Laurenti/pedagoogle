"use client";
import { MdClose, MdWarning } from "react-icons/md";

interface ModalProps {
  open: boolean;
  onClose: () => void;
  title?: string;
  variant?: "form" | "confirm" | "info";
  color?: "primary" | "error" | "warning" | "success" | "info";
  confirmLabel?: string;
  cancelLabel?: string;
  onConfirm?: () => void;
  children?: React.ReactNode;
  size?: "sm" | "md" | "lg" | "xl";
}

const sizeClass = { sm: "max-w-sm", md: "max-w-md", lg: "max-w-lg", xl: "max-w-xl" } as const;
const btnColor  = { primary: "btn-primary", error: "btn-error", warning: "btn-warning", success: "btn-success", info: "btn-info" } as const;
const txtColor  = { primary: "text-primary", error: "text-error", warning: "text-warning", success: "text-success", info: "text-info" } as const;

export default function Modal({
  open,
  onClose,
  title,
  variant = "form",
  color,
  confirmLabel = "Confirmar",
  cancelLabel = "Cancelar",
  onConfirm,
  children,
  size = "md",
}: ModalProps) {
  if (!open) return null;

  const c = color ?? (variant === "confirm" ? "error" : "primary");

  return (
    <div className="modal modal-open">
      <div className={`modal-box ${sizeClass[size]} ${variant === "form" ? "overflow-y-auto max-h-[90vh]" : ""}`}>
        <div className="flex items-center justify-between mb-4">
          {title && <h3 className="font-bold text-lg">{title}</h3>}
          <button className="btn btn-sm btn-ghost ml-auto" type="button" onClick={onClose}>
            <MdClose size={18} />
          </button>
        </div>

        {variant === "confirm" && (
          <>
            <div className="flex items-start gap-3 mb-6">
              <MdWarning size={24} className={`${txtColor[c]} shrink-0 mt-0.5`} />
              <div className="text-sm">{children}</div>
            </div>
            <div className="modal-action">
              <button className="btn" type="button" onClick={onClose}>{cancelLabel}</button>
              <button className={`btn ${btnColor[c]}`} type="button" onClick={onConfirm}>{confirmLabel}</button>
            </div>
          </>
        )}

        {variant === "info" && (
          <>
            <div className="mb-6">{children}</div>
            <div className="modal-action">
              <button className="btn" type="button" onClick={onClose}>Fechar</button>
            </div>
          </>
        )}

        {variant === "form" && children}
      </div>
      <div className="modal-backdrop" onClick={onClose} />
    </div>
  );
}
