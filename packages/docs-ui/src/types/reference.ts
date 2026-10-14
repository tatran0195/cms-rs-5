import type React from 'react';
import type { FrontMatter } from './frontmatter.js';
import type { Workflow } from './workflow.js';

export type Ref<T = any> = React.Ref<T>;

export type DocTypeListItem = {
  name: string;
  type: string;
  optional?: boolean;
  defaultValue?: string;
  example?: string;
  description?: string;
  featureFlag?: string;
  expandable: boolean;
  children?: DocTypeListItem[];
  since?: string;
  deprecated?: {
    is_deprecated: boolean;
    description?: string;
  };
};

export type DocCodeTab = {
  label: string;
  language: string;
  title?: string;
  code: string;
};

export type DocWorkflowEvent = {
  name: string;
  description?: string;
  payload?: string;
  deprecated?: boolean;
  deprecatedMessage?: string;
  since?: string;
};

export type DocEvent = {
  name: string;
  id: string;
  description?: string;
  payload?: string;
  workflows?: { name: string; href: string }[];
  deprecated?: boolean;
  deprecatedMessage?: string;
  since?: string;
};

export type DocBlock =
  | { kind: 'markdown'; html: string }
  | { kind: 'heading'; level: number; text: string; id: string }
  | {
      kind: 'typeList';
      sectionTitle?: string;
      expandUrl?: string;
      openedLevel?: number;
      types: DocTypeListItem[];
    }
  | { kind: 'workflowDiagram'; workflow: Workflow }
  | { kind: 'codeTabs'; tabs: DocCodeTab[] }
  | { kind: 'note'; variant?: string; html: string; title?: string }
  | { kind: 'sourceCodeLink'; link: string; text?: string }
  | { kind: 'table'; headers: string[]; rows: string[][] }
  | {
      kind: 'linkList';
      items: { title: string; href: string; description?: string }[];
    }
  | {
      kind: 'badges';
      badges: { variant?: string; label: string; tooltip?: string }[];
    }
  | { kind: 'workflowEvents'; events: DocWorkflowEvent[] }
  | {
      kind: 'eventsListing';
      categories: { title?: string; events: DocEvent[] }[];
    };

export type DocPage = {
  slug: string;
  title: string;
  frontmatter: FrontMatter;
  toc?: {
    title: string;
    id: string;
    level: number;
  }[];
  blocks: DocBlock[];
};
