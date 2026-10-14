"use client";

import type { MenuItemLink, MenuItem as MenuItemType } from "@/types";
import clsx from "clsx";
import Link from "next/link";

export type MenuItemProps = {
  item: MenuItemLink;
  onClick?: (item: MenuItemType) => void;
};

export const MenuItem = ({ item, onClick }: MenuItemProps) => (
  <div className="px-docs_0.25">
    <Link
      className={clsx(
        "flex py-docs_0.25 px-docs_0.5",
        "gap-docs_0.5 rounded-docs_xs",
        "hover:bg-cms-bg-component-hover",
        "text-cms-fg-base",
      )}
      href={item.link}
      onClick={() => onClick?.(item)}
      target={item.openInNewTab ? "_blank" : undefined}
      rel={item.openInNewTab ? "noopener noreferrer" : undefined}
    >
      {item.icon && (
        <span className="text-cms-fg-subtle mt-[2.5px] block">{item.icon}</span>
      )}
      <span className="text-compact-small">{item.title}</span>
    </Link>
  </div>
);
