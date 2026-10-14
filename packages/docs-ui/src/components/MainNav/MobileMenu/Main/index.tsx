'use client';

import { TriangleRightMini } from '@cms/icons';
import clsx from 'clsx';
import { Link } from '@/components/Link';
import type React from 'react';
import { useMainNav } from '../../../../providers/MainNav';
import type { SelectedMenu } from '..';

type MainNavMobileMainMenu = {
  setSelectedMenus: React.Dispatch<React.SetStateAction<SelectedMenu>>;
  onOpenLink?: () => void;
};

export const MainNavMobileMainMenu = ({ setSelectedMenus: setSelectedMenu, onOpenLink }: MainNavMobileMainMenu) => {
  const { navItems } = useMainNav();

  return (
    <div className="flex flex-col gap-[23px]">
      <span className="text-compact-small-plus text-cms-fg-muted uppercase">Menu</span>
      <ul className="flex flex-col gap-[18px]">
        {navItems.map((item, index) => (
          <li
            key={index}
            className={clsx('text-h1 text-cms-fg-base cursor-pointer', 'flex justify-between gap-docs_1')}
            onClick={() => {
              if (item.type !== 'dropdown') {
                return;
              }

              setSelectedMenu((prev) => [
                ...prev,
                {
                  title: item.title,
                  menu: item.children,
                },
              ]);
            }}
          >
            {item.type === 'link' && (
              <Link href={item.link} className="block w-full" onClick={() => onOpenLink?.()}>
                {item.title}
              </Link>
            )}
            {item.type === 'dropdown' && (
              <>
                <span>{item.title}</span>
                <TriangleRightMini data-testid="triangle-icon" />
              </>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
};
