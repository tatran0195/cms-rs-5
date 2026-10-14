"use client";

import type { Sidebar } from "@/types";
import type React from "react";
import { DottedSeparator } from "@/components/DottedSeparator";
import { SidebarItemCategory } from "./Category";
import { SidebarItemLink } from "./Link";
import { SidebarItemSidebar } from "./Sidebar";
import { SidebarItemSubCategory } from "./SubCategory";

export type SidebarItemProps = {
  item: Sidebar.SidebarItem;
  nested?: boolean;
  hasNextItems?: boolean;
  isParentCategoryOpen?: boolean;
} & React.AllHTMLAttributes<HTMLElement>;

export const SidebarItem = ({
  item,
  hasNextItems = false,
  ...props
}: SidebarItemProps) => {
  switch (item.type) {
    case "category":
      return (
        <>
          <SidebarItemCategory item={item} {...props} />
          {hasNextItems && <DottedSeparator />}
        </>
      );
    case "sub-category":
      return <SidebarItemSubCategory item={item} {...props} />;
    case "link":
    case "ref":
    case "external":
      return <SidebarItemLink item={item} {...props} />;
    case "sidebar":
      return <SidebarItemSidebar item={item} {...props} />;
    case "separator":
      return <DottedSeparator />;
  }
};
