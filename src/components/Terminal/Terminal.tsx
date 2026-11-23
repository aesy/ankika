import type { FC, ReactNode } from 'react';
import { bar, dot, frame, title as titleClass } from '@/components/Terminal/Terminal.css';

type Props = {
  title: string;
  actions?: ReactNode;
  className?: string;
  children: ReactNode;
};

/** A terminal window in the TUI's dark palette, whatever the page's color scheme. */
export const Terminal: FC<Props> = ({ title, actions, className, children }) => (
  <div className={className ? `${frame} ${className}` : frame}>
    <div className={bar}>
      <i className={dot} />
      <i className={dot} />
      <i className={dot} />
      <span className={titleClass}>{title}</span>
      {actions}
    </div>
    {children}
  </div>
);
