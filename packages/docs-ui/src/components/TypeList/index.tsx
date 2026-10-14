import clsx from 'clsx';
import type React from 'react';
import TypeListItems from './Items';

export type CommonProps = {
  expandUrl?: string;
  sectionTitle?: string;
  openedLevel?: number;
};

export type Type = {
  name: string;
  type: string;
  optional?: boolean;
  defaultValue?: string;
  example?: string;
  description?: string;
  featureFlag?: string;
  expandable: boolean;
  children?: Type[];
  deprecated?: {
    is_deprecated: boolean;
    description?: string;
  };
  since?: string;
};

type ParameterTypesType = {
  types: Type[];
  sectionTitle?: string;
} & CommonProps &
  React.HTMLAttributes<HTMLDivElement>;

export const TypeList = ({ types, className, sectionTitle, expandUrl, openedLevel, ...props }: ParameterTypesType) => (
  <div
    className={clsx('bg-cms-bg-subtle rounded my-docs_1', 'shadow-elevation-card-rest dark:shadow-elevation-card-rest-dark', className)}
    {...props}
  >
    <TypeListItems types={types} expandUrl={expandUrl} sectionTitle={sectionTitle} openedLevel={openedLevel} />
  </div>
);
