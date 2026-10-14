import { render } from '@testing-library/react';
import { describe, expect, test } from 'vitest';
import { Link } from '../../Link';

describe('rendering', () => {
  test('renders link with href', () => {
    const { container } = render(<Link href="/test">Test Link</Link>);
    const link = container.querySelector('a');
    expect(link).toBeInTheDocument();
    expect(link).toHaveAttribute('href', '/test');
    expect(link).toHaveTextContent('Test Link');
  });

  test('renders link with to prop', () => {
    const { container } = render(<Link to="/test-to">Test To</Link>);
    const link = container.querySelector('a');
    expect(link).toBeInTheDocument();
    expect(link).toHaveAttribute('href', '/test-to');
    expect(link).toHaveTextContent('Test To');
  });

  test('renders external link with target and rel', () => {
    const { container } = render(<Link href="https://example.com">External</Link>);
    const link = container.querySelector('a');
    expect(link).toBeInTheDocument();
    expect(link).toHaveAttribute('href', 'https://example.com');
    expect(link).toHaveAttribute('target', '_blank');
    expect(link).toHaveAttribute('rel', 'noopener noreferrer');
  });

  test('renders with default variant', () => {
    const { container } = render(<Link href="/test">Test Link</Link>);
    const link = container.querySelector('a');
    expect(link).toHaveClass('text-cms-fg-interactive hover:text-cms-fg-interactive-hover');
  });

  test('renders with content variant', () => {
    const { container } = render(
      <Link href="/test" variant="content">
        Test Link
      </Link>,
    );
    const link = container.querySelector('a');
    expect(link).toHaveClass('underline decoration-cms-fg-muted hover:decoration-cms-fg-interactive');
    expect(link).toHaveClass('decoration-[1.5px] font-medium transition-[text-decoration-color]');
  });

  test('applies custom className', () => {
    const { container } = render(
      <Link href="/test" className="custom-class">
        Test Link
      </Link>,
    );
    const link = container.querySelector('a');
    expect(link).toHaveClass('custom-class');
  });

  test('renders with icon when withIcon is true', () => {
    const { container } = render(
      <Link href="/test" withIcon>
        Test Link
      </Link>,
    );
    const link = container.querySelector('a');
    expect(link).toHaveClass('flex gap-0.25 items-center group');
    const icon = container.querySelector('svg');
    expect(icon).toBeInTheDocument();
  });

  test('does not render icon when withIcon is false', () => {
    const { container } = render(<Link href="/test">Test Link</Link>);
    const icon = container.querySelector('svg');
    expect(icon).not.toBeInTheDocument();
  });

  test('transforms page.mdx href', () => {
    const { container } = render(<Link href="/test/page.mdx">Test Link</Link>);
    const link = container.querySelector('a');
    expect(link).toHaveAttribute('href', '/test');
  });

  test('transforms page.mdx href with hash', () => {
    const { container } = render(<Link href="/test/page.mdx#section">Test Link</Link>);
    const link = container.querySelector('a');
    expect(link).toHaveAttribute('href', '/test#section');
  });

  test('handles empty href', () => {
    const { container } = render(<Link href="">Test Link</Link>);
    const link = container.querySelector('a');
    expect(link).toHaveAttribute('href', '');
  });

  test('handles undefined href', () => {
    const { container } = render(<Link>Test Link</Link>);
    const link = container.querySelector('a');
    expect(link).toHaveAttribute('href', '');
  });

  test('passes through other props', () => {
    const { container } = render(
      <Link href="/test" target="_blank" rel="noopener">
        Test Link
      </Link>,
    );
    const link = container.querySelector('a');
    expect(link).toHaveAttribute('target', '_blank');
    expect(link).toHaveAttribute('rel', 'noopener');
  });
});
