"use client";

// @refresh reset

import type { Sidebar } from "@/types";
import clsx from "clsx";
import Link from "next/link";
import type React from "react";
import { useMemo } from "react";
import { Badge } from "@/components/Badge";
import { SidebarItemTitle } from "@/components/Sidebar/Item/Title";
import { useSidebar } from "@/providers/Sidebar";

export type SidebarItemSidebarProps = {
  item: Sidebar.SidebarItemSidebar;
  nested?: boolean;
} & React.AllHTMLAttributes<HTMLLIElement>;

export const SidebarItemSidebar = ({
  item,
  className,
  nested = false,
}: SidebarItemSidebarProps) => {
  const { getSidebarFirstLinkChild: getSidebarFirstChild } = useSidebar();

  const isTitleOneWord = useMemo(
    () => item.title.split(" ").length === 1,
    [item.title],
  );

  const firstChild = useMemo(
    () => getSidebarFirstChild(item),
    [item, getSidebarFirstChild],
  );

  return (
    <li>
      <span className="block px-docs_0.75">
        <Link
          href={firstChild?.path ?? "#"}
          className={clsx(
            "py-docs_0.25 px-docs_0.5",
            "block w-full rounded-docs_sm",
            !isTitleOneWord && "break-words",
            !nested && "text-cms-fg-subtle",
            nested && "text-cms-fg-muted",
            "hover:bg-cms-bg-base-hover lg:hover:bg-cms-bg-subtle-hover",
            "text-compact-small-plus",
            "flex justify-between items-center gap-[6px]",
            className,
          )}
          {...firstChild?.linkProps}
        >
          <SidebarItemTitle
            title={item.title}
            isTitleOneWord={isTitleOneWord}
            className={clsx(nested && "inline-block pl-docs_1.5")}
          />
          {item.additionalElms}
          {item.badge && (
            <Badge variant={item.badge.variant}>{item.badge.text}</Badge>
          )}
        </Link>
      </span>
    </li>
  );
};
