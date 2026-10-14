"use client";

import type { MenuItem, MenuItemAction } from "../../types";
import clsx from "clsx";

export type MenuActionProps = {
  item: MenuItemAction;
  onClick?: (item: MenuItem) => void;
};

export const MenuAction = ({ item, onClick }: MenuActionProps) => (
  <div className="px-docs_0.25">
    <span
      className={clsx(
        "flex py-docs_0.25 px-docs_0.5",
        "gap-docs_0.5 rounded-docs_xs",
        "hover:bg-cms-bg-component-hover",
        "text-cms-fg-base cursor-pointer",
      )}
      tabIndex={-1}
      onClick={() => {
        item.action();
        onClick?.(item);
      }}
      data-testid="menu-action"
    >
      <span className="text-cms-fg-subtle mt-[2.5px] block">{item.icon}</span>
      <span className="text-compact-small flex-1">{item.title}</span>
      {item.shortcut && (
        <span
          className="text-cms-fg-subtle text-compact-small"
          data-testid="menu-action-shortcut"
        >
          {item.shortcut}
        </span>
      )}
    </span>
  </div>
);
