import { render } from '@testing-library/react';
import { afterEach, describe, expect, test } from 'vitest';
import { BareboneLayout } from '../barebone';

describe('BareboneLayout', () => {
  afterEach(() => {
    document.querySelectorAll('script').forEach((s) => {
      s.remove();
    });
  });
  test('renders children in a container div', () => {
    const { getByText } = render(
      <BareboneLayout>
        <div>content</div>
      </BareboneLayout>,
    );
    expect(getByText('content')).toBeInTheDocument();
  });

  test('renders Google Analytics script when gaId is provided', () => {
    render(
      <BareboneLayout gaId="G-TESTID">
        <div>content</div>
      </BareboneLayout>,
    );
    const script = document.querySelector('script');
    expect(script).toBeInTheDocument();
    expect(script).toHaveAttribute('src', 'https://www.googletagmanager.com/gtag/js?id=G-TESTID');
  });

  test('does not render script when gaId is not provided', () => {
    render(
      <BareboneLayout>
        <div>content</div>
      </BareboneLayout>,
    );
    const script = document.querySelector('script');
    expect(script).not.toBeInTheDocument();
  });
});
