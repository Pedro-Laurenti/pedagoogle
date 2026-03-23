'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { useCurrentPage } from '@/hooks/useCurrentPage';
import { menuItems, MenuItem } from '@/lib/navigation';
import { MdExpandMore, MdKeyboardDoubleArrowRight, MdDashboard } from 'react-icons/md';
import ThemeToggle from '@/components/ThemeToggle';
import packageJson from '@/package.json';
import { invokeCmd } from '@/utils/tauri';
import type { Configuracoes } from '@/types';

export default function Sidebar() {
  const pathname = useCurrentPage();
  const [isCollapsed, setIsCollapsed] = useState(false);
  const [expandedItems, setExpandedItems] = useState<Set<string>>(new Set());
  const [config, setConfig] = useState<Configuracoes | null>(null);

  useEffect(() => {
    invokeCmd<Configuracoes>('get_configuracoes').then(setConfig).catch(() => {});
  }, []);

  useEffect(() => {
    const saved = localStorage.getItem('sidebar-collapsed');
    if (saved !== null) {
      setIsCollapsed(saved === 'true');
    }

    const currentItem = menuItems.find(
      (item) => item.href && (pathname === item.href || pathname.startsWith(item.href + '/'))
    );
    if (currentItem?.parentHref) {
      setExpandedItems((prev) => new Set(prev).add(currentItem.parentHref!));
    }
  }, [pathname]);

  const toggleCollapse = () => {
    const newState = !isCollapsed;
    setIsCollapsed(newState);
    localStorage.setItem('sidebar-collapsed', String(newState));
  };

  const getItemId = (item: MenuItem): string | undefined => item.id || item.href;

  const toggleExpanded = (itemId: string) => {
    setExpandedItems((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(itemId)) {
        newSet.delete(itemId);
      } else {
        newSet.add(itemId);
      }
      return newSet;
    });
  };

  const closeMobileDrawer = () => {
    const drawerToggle = document.getElementById('drawer-toggle') as HTMLInputElement;
    if (drawerToggle) drawerToggle.checked = false;
  };

  const sidebarItems = menuItems.filter((item) => {
    if (item.showInSidebar === false) return false;
    if (item.feature && config && !config[item.feature]) return false;
    return true;
  });

  const organizedItems = sidebarItems.reduce((acc, item) => {
    const group = item.group || '';
    if (!acc[group]) acc[group] = [];

    if (
      !item.parentHref ||
      !sidebarItems.find((m) => {
        const mId = getItemId(m);
        return mId && mId === item.parentHref && m.showInSidebar !== false;
      })
    ) {
      acc[group].push(item);
    }

    return acc;
  }, {} as Record<string, MenuItem[]>);

  const getChildren = (parentId?: string): MenuItem[] => {
    if (!parentId) return [];
    return sidebarItems.filter(
      (item) => item.parentHref === parentId && item.showInSidebar !== false
    );
  };

  const renderMenuItem = (item: MenuItem, isChild = false): React.ReactNode => {
    const IconComponent = item.icon;
    const isActive = item.href ? pathname === item.href : false;
    const itemId = getItemId(item);
    const children = getChildren(itemId);
    const hasChildren = children.length > 0;
    const isExpanded = itemId ? expandedItems.has(itemId) : false;
    const isChildActive = children.some(
      (child) => child.href && (pathname === child.href || pathname.startsWith(child.href + '/'))
    );

    return (
      <li key={itemId || item.label}>
        {hasChildren ? (
          <>
            <div
              className={`flex items-center gap-3 p-3 rounded-lg transition-all duration-200 cursor-pointer ${
                isActive || isChildActive
                  ? 'bg-primary text-primary-content shadow-lg'
                  : 'hover:bg-base-300 text-base-content/80 hover:text-base-content'
              }`}
              onClick={() => {
                if (isCollapsed) {
                  if (itemId) toggleExpanded(itemId);
                } else {
                  if (item.href) {
                    closeMobileDrawer();
                    window.location.href = item.href;
                  }
                }
              }}
            >
              <IconComponent
                className={`w-5 h-5 shrink-0 ${isActive || isChildActive ? 'text-primary-content' : 'text-base-content/60'}`}
              />
              {!isCollapsed && (
                <>
                  <div className="flex-1">
                    <div className="font-medium">{item.label}</div>
                    {item.description && !isActive && !isChildActive && (
                      <div className="text-xs text-base-content/50 mt-0.5 line-clamp-1">
                        {item.description}
                      </div>
                    )}
                  </div>
                  <button
                    className="p-2 -m-2 hover:bg-base-content/10 hover:cursor-pointer rounded-lg transition-colors"
                    onClick={(e) => {
                      e.stopPropagation();
                      if (itemId) toggleExpanded(itemId);
                    }}
                  >
                    <MdExpandMore
                      className={`w-5 h-5 transition-transform duration-200 border border-base-content/30 rounded-full ${isExpanded ? 'rotate-180' : ''}`}
                    />
                  </button>
                </>
              )}
            </div>
            {isExpanded && !isCollapsed && (
              <ul className="ml-4 mt-1 space-y-1">
                {children.map((child) => renderMenuItem(child, true))}
              </ul>
            )}
          </>
        ) : item.href ? (
          <Link
            href={item.href}
            prefetch={false}
            onClick={closeMobileDrawer}
            className={`flex items-center gap-3 p-3 rounded-lg transition-all duration-200 ${
              isCollapsed ? 'justify-center tooltip tooltip-right' : ''
            } ${isChild ? 'pl-4' : ''} ${
              isActive
                ? 'bg-primary text-primary-content shadow-lg'
                : 'hover:bg-base-300 text-base-content/80 hover:text-base-content'
            }`}
            data-tip={isCollapsed ? item.label : undefined}
          >
            <IconComponent
              className={`w-5 h-5 shrink-0 ${isActive ? 'text-primary-content' : 'text-base-content/60'}`}
            />
            {!isCollapsed && (
              <>
                <div className="flex-1">
                  <div className="font-medium">{item.label}</div>
                  {item.description && !isActive && (
                    <div className="text-xs text-base-content/50 mt-0.5 line-clamp-1">
                      {item.description}
                    </div>
                  )}
                </div>
                {isActive && <div className="w-2 h-2 bg-primary-content rounded-full" />}
              </>
            )}
          </Link>
        ) : (
          <div
            className={`flex items-center gap-3 p-3 rounded-lg transition-all duration-200 cursor-default ${
              isCollapsed ? 'justify-center tooltip tooltip-right' : ''
            } ${isChild ? 'pl-4' : ''} text-base-content/80`}
            data-tip={isCollapsed ? item.label : undefined}
          >
            <IconComponent className="w-5 h-5 shrink-0 text-base-content/60" />
            {!isCollapsed && (
              <div className="flex-1">
                <div className="font-medium">{item.label}</div>
                {item.description && (
                  <div className="text-xs text-base-content/50 mt-0.5 line-clamp-1">
                    {item.description}
                  </div>
                )}
              </div>
            )}
          </div>
        )}
      </li>
    );
  };

  return (
    <>
      <aside
        className={`bg-base-200 h-screen flex flex-col border-r border-base-300 transition-all duration-300 ${isCollapsed ? 'w-20' : 'w-80'}`}
      >
        <div className="p-6 border-b border-base-300 shrink-0">
          <div className={`flex items-center ${isCollapsed ? 'justify-center' : 'gap-3'}`}>
            <div className="w-10 h-10 bg-primary rounded-lg flex items-center justify-center shrink-0">
              <MdDashboard className="text-primary-content text-lg" />
            </div>
            {!isCollapsed && (
              <div>
                <h2 className="text-lg font-bold">Pedagoogle</h2>
                <p className="text-xs text-base-content/60">Gestao Escolar</p>
              </div>
            )}
          </div>
        </div>

        <nav className="flex-1 p-4 overflow-y-auto overflow-x-hidden min-h-0 scrollbar-thin">
          {Object.entries(organizedItems).map(([groupName, items]) => (
            <div key={groupName} className="mb-6">
              {!isCollapsed && groupName && (
                <div className="text-xs font-semibold text-base-content/60 uppercase tracking-wider mb-3 px-2">
                  {groupName}
                </div>
              )}
              <ul className="menu p-0 w-full space-y-1">
                {items.map((item) => renderMenuItem(item))}
              </ul>
            </div>
          ))}
        </nav>

        <div className="p-4 border-t border-base-300 shrink-0">
          <div className={`flex ${isCollapsed ? 'flex-col items-center gap-2' : 'gap-2 items-center'}`}>
            <ThemeToggle />
            {!isCollapsed && (
              <div className="text-xs text-base-content/40">v{packageJson.version}</div>
            )}
          </div>
        </div>
      </aside>

      <button
        onClick={toggleCollapse}
        className="hidden lg:block fixed bottom-1/2 z-20 bg-base-200 border border-base-300 p-2 rounded-full transition-all duration-300 hover:cursor-pointer hover:bg-base-300"
        style={{ left: isCollapsed ? '4rem' : '19rem' }}
      >
        <MdKeyboardDoubleArrowRight
          className={`w-4 h-4 transition-transform duration-300 ${isCollapsed ? '' : 'rotate-180'}`}
        />
      </button>
    </>
  );
}
