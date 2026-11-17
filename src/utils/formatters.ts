// Times for display, in the app's language (the browser's Intl does the wording: "5 minutes ago",
// "5 分鐘前", "yesterday", "昨天").
import i18n from '../i18n';

const UNITS: Array<[Intl.RelativeTimeFormatUnit, number]> = [
  ['year', 365 * 86400],
  ['month', 30 * 86400],
  ['week', 7 * 86400],
  ['day', 86400],
  ['hour', 3600],
  ['minute', 60],
];

/** How long ago `isoString` was ("now", "5 minutes ago", "yesterday"). */
export function formatRelativeTime(isoString: string, lang: string = i18n.language, now: number = Date.now()): string {
  const seconds = Math.floor((now - new Date(isoString).getTime()) / 1000);
  if (!isFinite(seconds)) return '--';
  const format = new Intl.RelativeTimeFormat(lang, { numeric: 'auto' });
  for (const [unit, size] of UNITS) {
    if (seconds >= size) return format.format(-Math.floor(seconds / size), unit);
  }
  return format.format(0, 'second');
}

/** Date and time of `isoString` ("2025/12/03 14:30", "12/03/2025, 2:30 PM"). */
export function formatAbsoluteTime(isoString: string, lang: string = i18n.language): string {
  const date = new Date(isoString);
  if (isNaN(date.getTime())) return '--';
  return date.toLocaleString(lang, { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' });
}

/** A duration in at most two units ("1h 15m", "2分鐘 30秒"); "--" when unknown. */
export function formatDuration(seconds: number | null, lang: string = i18n.language): string {
  if (seconds === null || !isFinite(seconds) || seconds < 0) return '--';
  const unit = (value: number, name: 'hour' | 'minute' | 'second') =>
    new Intl.NumberFormat(lang, { style: 'unit', unit: name, unitDisplay: 'narrow' }).format(value);
  const [h, m, s] = [Math.floor(seconds / 3600), Math.floor((seconds % 3600) / 60), Math.floor(seconds % 60)];
  if (h > 0) return `${unit(h, 'hour')} ${unit(m, 'minute')}`;
  if (m > 0) return `${unit(m, 'minute')} ${unit(s, 'second')}`;
  return unit(s, 'second');
}
