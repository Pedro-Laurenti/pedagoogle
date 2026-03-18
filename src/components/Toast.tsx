"use client";

interface ToastProps {
  message: string | null;
  type?: "success" | "error" | "warning" | "info";
  onClose: () => void;
}

export default function Toast({ message, type = "success", onClose }: ToastProps) {
  if (!message) return null;
  return (
    <div className="toast toast-end">
      <div className={`alert alert-${type}`}>
        <span>{message}</span>
        <button className="btn btn-xs btn-ghost" onClick={onClose}>x</button>
      </div>
    </div>
  );
}
