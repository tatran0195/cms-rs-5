import clsx from 'clsx';
import type React from 'react';

export type BareboneLayoutProps = {
  htmlClassName?: string;
  children: React.ReactNode;
  gaId?: string;
};

export const BareboneLayout = ({ htmlClassName, children, gaId }: BareboneLayoutProps) => (
  <div className={clsx('h-full w-full', htmlClassName)}>
    {children}
    {gaId && (
      <script
        async
        src={`https://www.googletagmanager.com/gtag/js?id=${gaId}`}
      />
    )}
  </div>
);
