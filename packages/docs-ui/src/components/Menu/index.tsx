import type { MenuItem as MenuItemType } from "../../types";
import clsx from "clsx";
import React from "react";
import { MenuAction } from "./Action";
import { MenuDivider } from "./Divider";
import { MenuItem } from "./Item";
import { MenuSubMenu } from "./SubMenu";

export type MenuProps = {
  items: MenuItemType[];
  className?: string;
  itemsOnClick?: (item: MenuItemType) => void;
};

export const Menu = ({ items, className, itemsOnClick }: MenuProps) => (
  <div
    className={clsx(
      "bg-cms-bg-component py-docs_0.25 rounded-docs_DEFAULT",
      "shadow-elevation-flyout dark:shadow-elevation-flyout-dark",
      className,
    )}
    data-testid="menu"
  >
    {items.map((item, index) => (
      <React.Fragment key={index}>
        {item.type === "link" && (
          <MenuItem item={item} onClick={itemsOnClick} />
        )}
        {item.type === "action" && (
          <MenuAction item={item} onClick={itemsOnClick} />
        )}
        {item.type === "divider" && <MenuDivider />}
        {item.type === "custom" && item.content}
        {item.type === "sub-menu" && (
          <MenuSubMenu item={item} itemsOnClick={itemsOnClick} />
        )}
      </React.Fragment>
    ))}
  </div>
);

export * from "./Dropdown";
