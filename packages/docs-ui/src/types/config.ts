import type { Sidebar } from './sidebar.js';

export type BreadcrumbOptions = {
  startItems?: {
    title: string;
    link: string;
  }[];
};

export type DocsConfig = {
  titleSuffix?: string;
  description?: string;
  baseUrl: string;
  basePath?: string;
  sidebars: Sidebar[];
  filesBasePath?: string;
  useNextLinks?: boolean;
  project: {
    title: string;
    key: string;
  };
  breadcrumbOptions?: BreadcrumbOptions;
  version: {
    number: string;
    releaseUrl: string;
    releaseDate: string;
    hide?: boolean;
    bannerImage?: {
      light: string;
      dark: string;
    };
  };
  reportIssueLink?: string;
  logo: string;
  features?: {
    aiAssistant?: boolean;
    [key: string]: any;
  };
  contentMenuSections?: Record<string, boolean>;
  [key: string]: any;
};
