import type { IconProps } from '@cms/icons';

export type NavigationDropdownItemLink = {
  path: string;
  isActive?: boolean;
  title: string;
  icon: React.FC<IconProps>;
};

export type NavigationDropdownItem =
  | ({
      type: 'link';
    } & NavigationDropdownItemLink)
  | {
      type: 'divider';
    };
