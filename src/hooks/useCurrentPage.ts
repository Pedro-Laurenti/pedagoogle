"use client";
import { usePathname } from "next/navigation";

export function useCurrentPage(): string {
  return usePathname();
}
