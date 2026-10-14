import type { IconProps } from '@cms/icons/dist/types';
import type { LinkProps } from '@/components/Link';
import type React from 'react';
import type { BadgeProps } from '@/components/Badge';
import { CardBloomLayout } from './Layout/Bloom';
import { CardDefaultLayout } from './Layout/Default';
import { CardFillerLayout } from './Layout/Filler';
import { CardLargeLayout } from './Layout/Large';
import { CardLayoutMini } from './Layout/Mini';

export type CardProps = {
  type?: 'default' | 'large' | 'filler' | 'mini' | 'bloom';
  icon?: React.FC<IconProps>;
  rightIcon?: React.FC<IconProps>;
  image?: string;
  themeImage?: {
    light: string;
    dark: string;
  };
  imageDimensions?: {
    width: number;
    height: number;
  };
  title?: string;
  text?: string;
  href?: string;
  className?: string;
  contentClassName?: string;
  iconClassName?: string;
  children?: React.ReactNode;
  badge?: BadgeProps;
  highlightText?: string[];
  closeable?: boolean;
  onClose?: () => void;
  onClick?: () => void;
  hrefProps?: Partial<LinkProps & React.AllHTMLAttributes<HTMLAnchorElement>>;
  cardRef?: React.Ref<HTMLDivElement>;
};

export const Card = ({ type = 'default', ...props }: CardProps) => {
  switch (type) {
    case 'large':
      return <CardLargeLayout {...props} />;
    case 'filler':
      return <CardFillerLayout {...props} />;
    case 'mini':
      return <CardLayoutMini {...props} />;
    case 'bloom':
      return <CardBloomLayout {...props} />;
    default:
      return <CardDefaultLayout {...props} />;
  }
};
