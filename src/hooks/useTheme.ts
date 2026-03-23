"use client";
import { useState, useEffect } from "react";
import { invokeCmd } from "@/utils/tauri";
import type { Configuracoes } from "@/types";

export function useTheme() {
  const [isDark, setIsDark] = useState(false);
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    invokeCmd<Configuracoes>("get_configuracoes")
      .then((c) => {
        const tema = c.tema || localStorage.getItem("theme") || "light";
        setIsDark(tema === "dark");
        document.documentElement.setAttribute("data-theme", tema);
      })
      .catch(() => {
        const saved = localStorage.getItem("theme") || "light";
        setIsDark(saved === "dark");
        document.documentElement.setAttribute("data-theme", saved);
      })
      .finally(() => setMounted(true));
  }, []);

  function toggleTheme() {
    const next = isDark ? "light" : "dark";
    setIsDark(!isDark);
    localStorage.setItem("theme", next);
    document.documentElement.setAttribute("data-theme", next);
  }

  return { toggleTheme, mounted, isDark };
}
