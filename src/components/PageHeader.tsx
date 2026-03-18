'use client';

import Link from 'next/link';
import { getPageInfo, getItemById } from '@/lib/navigation';
import { useCurrentPage } from '@/hooks/useCurrentPage';
import { MdHome } from 'react-icons/md';

export default function PageHeader() {
  const pathname = useCurrentPage();
  const currentPage = getPageInfo(pathname);

  if (!currentPage) {
    return null;
  }

  const buildBreadcrumbs = () => {
    const breadcrumbs: typeof currentPage[] = [];
    let current: typeof currentPage | undefined = currentPage;

    while (current) {
      breadcrumbs.unshift(current);
      if (current.parentHref) {
        current = getItemById(current.parentHref);
      } else {
        break;
      }
    }

    return breadcrumbs;
  };

  const breadcrumbs = buildBreadcrumbs();

  return (
    <div className="p-8 bg-base-100 shadow-sm border-b border-base-300">
      <div className="mx-auto">
        <div className="breadcrumbs text-xs md:text-sm mb-2">
          <ul>
            <li>
              <Link href="/dashboard" prefetch={false} className="flex items-center gap-1 hover:text-primary transition-colors">
                <MdHome />
                <span>Início</span>
              </Link>
            </li>
            {breadcrumbs.map((page, index) => {
              const isLast = index === breadcrumbs.length - 1;
              return (
                <li key={page.href || page.label}>
                  {isLast || !page.href ? (
                    <span className="text-base-content font-medium">{page.label}</span>
                  ) : (
                    <Link href={page.href} prefetch={false} className="hover:text-primary transition-colors">
                      {page.label}
                    </Link>
                  )}
                </li>
              );
            })}
          </ul>
        </div>
        <div className="flex items-center gap-3">
          {currentPage.icon && <currentPage.icon className="text-3xl text-primary" />}
          <div>
            <h1 className="text-2xl md:text-3xl font-bold text-base-content">{currentPage.label}</h1>
            {currentPage.description && (
              <p className="text-sm md:text-base text-base-content/70 mt-1">{currentPage.description}</p>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
