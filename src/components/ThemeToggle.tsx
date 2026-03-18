'use client';
import { FiSun, FiMoon } from 'react-icons/fi';
import { useTheme } from '@/hooks/useTheme';

export default function ThemeToggle() {
  const { toggleTheme, mounted, isDark } = useTheme();

  if (!mounted) {
    return <div className="opacity-0 btn btn-square btn-ghost" />;
  }

  return (
    <label className="swap swap-rotate btn btn-square btn-ghost">
      <input
        id="theme_toggle"
        type="checkbox"
        className="theme-controller"
        checked={isDark}
        onChange={toggleTheme}
        aria-label="Toggle theme"
      />
      <FiSun className="w-5 h-5 swap-off" />
      <FiMoon className="w-5 h-5 swap-on" />
    </label>
  );
}
