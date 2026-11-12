// What a failed task's error means, in the user's language.
import type { CoreError } from '../backend';

type T = (key: string, options?: Record<string, unknown>) => string;

/** A sentence for `error`; the backend's own detail is appended where it helps. */
export function errorText(error: CoreError | null, t: T): string {
  if (!error) return '';
  switch (error.kind) {
    case 'resolve':
      return error.detail.hint ? `${error.detail.message} (${error.detail.hint})` : error.detail.message;
    case 'http':
      return t('errors.core.http', { status: error.detail });
    case 'cancelled':
    case 'notFound':
      return t(`errors.core.${error.kind}`);
    default:
      return `${t(`errors.core.${error.kind}`)}: ${error.detail}`;
  }
}
