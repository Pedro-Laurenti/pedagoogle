import type { Metadata } from "next";
import "./globals.css";
import Sidebar from "@/components/Sidebar";

export const metadata: Metadata = {
  title: "Pedagoogle",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="pt-BR" suppressHydrationWarning>
      <head>
        <script
          dangerouslySetInnerHTML={{
            __html: `
              (function() {
                try {
                  var theme = localStorage.getItem('theme') || 'mylight';
                  document.documentElement.setAttribute('data-theme', theme);
                } catch (e) {
                  document.documentElement.setAttribute('data-theme', 'mylight');
                }
              })();
            `,
          }}
        />
      </head>
      <body className="antialiased bg-base-100" suppressHydrationWarning>
        <div className="drawer lg:drawer-open">
          <input id="drawer-toggle" type="checkbox" className="drawer-toggle" />
          <div className="drawer-content flex flex-col max-h-screen overflow-hidden">
            <div className="flex-1 overflow-y-auto">
              <main className="min-h-full">{children}</main>
            </div>
          </div>
          <div className="drawer-side">
            <label htmlFor="drawer-toggle" aria-label="Fechar menu lateral" className="drawer-overlay" />
            <Sidebar />
          </div>
        </div>
      </body>
    </html>
  );
}
