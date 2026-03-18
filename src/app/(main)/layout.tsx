import PageHeader from "@/components/PageHeader";

export default function SectionLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex flex-col min-h-full">
      <PageHeader />
      <div className="flex-1 p-8">{children}</div>
    </div>
  );
}
