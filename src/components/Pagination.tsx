"use client";
import { MdFirstPage, MdLastPage, MdChevronLeft, MdChevronRight } from "react-icons/md";

interface Props {
  page: number;
  total: number;
  perPage: number;
  onChange: (page: number) => void;
}

export default function Pagination({ page, total, perPage, onChange }: Props) {
  const totalPages = Math.max(1, Math.ceil(total / perPage));
  if (totalPages <= 1) return null;

  const pages: (number | "...")[] = [];
  for (let i = 1; i <= totalPages; i++) {
    if (i === 1 || i === totalPages || Math.abs(i - page) <= 1) {
      pages.push(i);
    } else if (pages[pages.length - 1] !== "...") {
      pages.push("...");
    }
  }

  return (
    <div className="join flex justify-center mt-4">
      <button className="join-item btn btn-sm" disabled={page <= 1} onClick={() => onChange(1)}>
        <MdFirstPage />
      </button>
      <button className="join-item btn btn-sm" disabled={page <= 1} onClick={() => onChange(page - 1)}>
        <MdChevronLeft />
      </button>
      {pages.map((p, i) =>
        p === "..." ? (
          <button key={`ellipsis-${i}`} className="join-item btn btn-sm btn-disabled">…</button>
        ) : (
          <button
            key={p}
            className={`join-item btn btn-sm ${p === page ? "btn-active" : ""}`}
            onClick={() => onChange(p)}
          >
            {p}
          </button>
        )
      )}
      <button className="join-item btn btn-sm" disabled={page >= totalPages} onClick={() => onChange(page + 1)}>
        <MdChevronRight />
      </button>
      <button className="join-item btn btn-sm" disabled={page >= totalPages} onClick={() => onChange(totalPages)}>
        <MdLastPage />
      </button>
    </div>
  );
}
