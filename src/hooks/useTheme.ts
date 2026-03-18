"use client";
import { useState, useEffect } from "react";

export function useTheme() {
  const [isDark, setIsDark] = useState(false);
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    const saved = localStorage.getItem("theme") || "mylight";
    setIsDark(saved === "mydark");
    document.documentElement.setAttribute("data-theme", saved);
    setMounted(true);
  }, []);

  function toggleTheme() {
    const next = isDark ? "mylight" : "mydark";
    setIsDark(!isDark);
    localStorage.setItem("theme", next);
    document.documentElement.setAttribute("data-theme", next);
  }

  return { toggleTheme, mounted, isDark };
}
