/** A component that throws shows an error with a reload button, not a blank window. */
import { describe, expect, test, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import ErrorBoundary from '../components/ErrorBoundary';
import i18n from '../i18n';

const Broken = () => {
  throw new Error('table exploded');
};

describe('error boundary', () => {
  test('shows what broke and offers a reload; the rest keeps working', async () => {
    await i18n.changeLanguage('zh-TW');
    vi.spyOn(console, 'error').mockImplementation(() => {});
    render(
      <>
        <ErrorBoundary>
          <Broken />
        </ErrorBoundary>
        <p>still here</p>
      </>,
    );
    expect(screen.getByText('這個畫面發生錯誤')).toBeInTheDocument();
    expect(screen.getByText('table exploded')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '重新載入' })).toBeInTheDocument();
    expect(screen.getByText('still here')).toBeInTheDocument();
  });
});
