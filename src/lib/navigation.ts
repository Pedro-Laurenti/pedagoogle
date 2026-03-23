import {
  MdDashboard,
  MdAssignment,
  MdPeople,
  MdSchool,
  MdGrade,
  MdCalendarToday,
  MdSettings,
  MdGroups,
  MdPersonOutline,
  MdFactCheck,
} from "react-icons/md";
import type { IconType } from "react-icons";

export interface MenuItem {
  id?: string;
  href?: string;
  label: string;
  icon: IconType;
  description?: string;
  group?: string;
  showInSidebar?: boolean;
  parentHref?: string;
  feature?: "usar_turmas" | "usar_professores" | "usar_frequencia" | "usar_recuperacao";
}

export const menuItems: MenuItem[] = [
  {
    href: "/dashboard",
    label: "Dashboard",
    icon: MdDashboard,
    description: "Visão geral do sistema",
    showInSidebar: true,
    group: "Escola",
  },
  {
    href: "/professores",
    label: "Professores",
    icon: MdPersonOutline,
    description: "Cadastro de professores",
    showInSidebar: true,
    feature: "usar_professores",
    group: "Pessoas",
  },
  {
    href: "/materias",
    label: "Matérias",
    icon: MdSchool,
    description: "Gerenciar matérias",
    showInSidebar: true,
    group: "Conteúdo",
  },
  {
    href: "/provas",
    label: "Provas",
    icon: MdAssignment,
    description: "Criar e gerenciar provas",
    showInSidebar: true,
    group: "Conteúdo",
  },
  {
    href: "/turmas",
    label: "Turmas",
    icon: MdGroups,
    description: "Gerenciar turmas",
    showInSidebar: true,
    feature: "usar_turmas",
    group: "Pessoas",
  },
  {
    href: "/alunos",
    label: "Alunos",
    icon: MdPeople,
    description: "Cadastro de alunos",
    showInSidebar: true,
    group: "Pessoas",
  },
  {
    href: "/notas",
    label: "Notas",
    icon: MdGrade,
    description: "Lançamento de notas",
    showInSidebar: true,
    group: "Avaliação",
  },
  {
    href: "/cronograma",
    label: "Cronograma",
    icon: MdCalendarToday,
    description: "Grade semanal de aulas",
    showInSidebar: true,
    group: "Avaliação",
  },
  {
    href: "/frequencia",
    label: "Frequência",
    icon: MdFactCheck,
    description: "Controle de presença",
    showInSidebar: true,
    feature: "usar_frequencia",
    group: "Avaliação",
  },
  {
    href: "/configuracoes",
    label: "Configurações",
    icon: MdSettings,
    description: "Dados da escola",
    showInSidebar: true,
    group: "Escola",
  },
];

export const createDynamicPageInfo = (
  basePath: string,
  label: string,
  description: string,
  parentHref: string
): MenuItem => {
  const parent = menuItems.find((item) => item.href === parentHref);
  const icon = parent?.icon || MdDashboard;
  return { href: basePath, label, icon, description, showInSidebar: false, parentHref };
};

export const getPageInfo = (pathname: string): MenuItem | undefined => {
  const exact = menuItems.find((item) => item.href === pathname);
  if (exact) return exact;

  const segments = pathname.split("/").filter(Boolean);
  if (segments.length >= 2) {
    const basePath = `/${segments.slice(0, -1).join("/")}`;
    const parent = menuItems.find((item) => item.href === basePath);
    if (parent) return createDynamicPageInfo(pathname, parent.label, parent.description || "", basePath);
  }

  return undefined;
};

export const getItemById = (idOrHref: string): MenuItem | undefined => {
  return menuItems.find(
    (item) => (item.id && item.id === idOrHref) || (item.href && item.href === idOrHref)
  );
};
