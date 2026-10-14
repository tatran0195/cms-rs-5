"use client";

import { BarsThree, Book, SidebarLeft, TimelineVertical } from "@cms/icons";
import type { MenuItem } from "@/types";
import clsx from "clsx";
import { useMemo, useRef, useState } from "react";
import { Button } from "@/components/Button";
import { Menu } from "@/components/Menu";
import { useClickOutside } from "@/hooks/use-click-outside";
import { useSidebar } from "@/providers/Sidebar";
import { getOsShortcut } from "@/utils/os-browser-utils";
import { useMainNav } from "../../../providers/MainNav";
import { HouseIcon } from "../../Icons/House";
import { MainNavThemeMenu } from "./ThemeMenu";

export const MainNavDesktopMenu = () => {
  const [isOpen, setIsOpen] = useState(false);
  const { setDesktopSidebarOpen, isSidebarShown, desktopSidebarOpen } =
    useSidebar();
  const { additionalMenuItems } = useMainNav();
  const ref = useRef<HTMLDivElement>(null);

  useClickOutside({
    elmRef: ref,
    onClickOutside: () => setIsOpen(false),
  });

  const items: MenuItem[] = useMemo(() => {
    const items: MenuItem[] = additionalMenuItems
      ? [...additionalMenuItems]
      : [
          {
            type: "link",
            icon: <HouseIcon />,
            title: "Homepage",
            link: "https://cms.com",
          },
          {
            type: "link",
            icon: <Book />,
            title: "CMS v1",
            link: "https://docs.cms.com/v1",
          },
          {
            type: "link",
            icon: <TimelineVertical />,
            title: "Changelog",
            link: "https://cms.com/changelog",
          },
        ];

    if (isSidebarShown) {
      items.push(
        {
          type: "divider",
        },
        {
          type: "action",
          title: desktopSidebarOpen ? "Hide Sidebar" : "Show Sidebar",
          icon: <SidebarLeft />,
          shortcut: `${getOsShortcut()}\\`,
          action: () => {
            setDesktopSidebarOpen((prev) => !prev);
            setIsOpen(false);
          },
        },
      );
    }

    items.push(
      {
        type: "divider",
      },
      {
        type: "custom",
        content: <MainNavThemeMenu />,
      },
    );

    return items;
  }, [
    isSidebarShown,
    desktopSidebarOpen,
    additionalMenuItems,
    setDesktopSidebarOpen,
  ]);

  return (
    <div
      className="relative hidden lg:flex justify-center items-center"
      ref={ref}
    >
      <Button
        variant="transparent"
        onClick={() => setIsOpen((prev) => !prev)}
        className="!p-[6.5px]"
        data-testid="menu-button"
      >
        <BarsThree className="text-cms-fg-subtle" />
      </Button>
      <Menu
        className={clsx(
          "absolute top-[calc(100%+8px)] right-0 min-w-[200px]",
          !isOpen && "hidden",
        )}
        items={items}
      />
    </div>
  );
};
