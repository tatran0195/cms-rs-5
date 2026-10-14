import { render } from '@testing-library/react';
import { describe, expect, test } from 'vitest';
import { LinkButton } from '../../LinkButton';

describe('rendering', () => {
  test('renders link button with href', () => {
    const { container } = render(<LinkButton href="/test">Test Button</LinkButton>);
    const link = container.querySelector('a');
    expect(link).toBeInTheDocument();
    expect(link).toHaveAttribute('href', '/test');
    expect(link).toHaveTextContent('Test Button');
  });

  test('renders with base variant by default', () => {
    const { container } = render(<LinkButton href="/test">Test Button</LinkButton>);
    const link = container.querySelector('a');
    expect(link).toHaveClass('text-cms-fg-base hover:text-cms-fg-subtle focus:text-cms-fg-base');
  });

  test('renders with interactive variant', () => {
    const { container } = render(
      <LinkButton href="/test" variant="interactive">
        Test Button
      </LinkButton>,
    );
    const link = container.querySelector('a');
    expect(link).toHaveClass('text-cms-fg-interactive hover:text-cms-interactive-hover focus:text-cms-fg-interactive');
  });

  test('renders with subtle variant', () => {
    const { container } = render(
      <LinkButton href="/test" variant="subtle">
        Test Button
      </LinkButton>,
    );
    const link = container.querySelector('a');
    expect(link).toHaveClass('text-cms-fg-subtle hover:text-cms-fg-base focus:text-cms-fg-subtle');
  });

  test('renders with muted variant', () => {
    const { container } = render(
      <LinkButton href="/test" variant="muted">
        Test Button
      </LinkButton>,
    );
    const link = container.querySelector('a');
    expect(link).toHaveClass('text-cms-fg-muted hover:text-cms-fg-subtle focus:text-cms-fg-muted');
  });

  test('applies custom className', () => {
    const { container } = render(
      <LinkButton href="/test" className="custom-class">
        Test Button
      </LinkButton>,
    );
    const link = container.querySelector('a');
    expect(link).toHaveClass('custom-class');
  });

  test('passes through other props', () => {
    const { container } = render(
      <LinkButton href="/test" target="_blank" rel="noopener">
        Test Button
      </LinkButton>,
    );
    const link = container.querySelector('a');
    expect(link).toHaveAttribute('target', '_blank');
    expect(link).toHaveAttribute('rel', 'noopener');
  });
});
