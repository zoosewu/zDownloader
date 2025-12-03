// Formatting utilities for display

/**
 * Format ISO 8601 timestamp to relative time (e.g., "5 minutes ago", "yesterday")
 * @param isoString ISO 8601 timestamp string
 * @returns Human-readable relative time string
 */
export function formatRelativeTime(isoString: string): string {
  try {
    const date = new Date(isoString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffSeconds = Math.floor(diffMs / 1000);
    const diffMinutes = Math.floor(diffSeconds / 60);
    const diffHours = Math.floor(diffMinutes / 60);
    const diffDays = Math.floor(diffHours / 24);

    // Just now (< 1 minute)
    if (diffSeconds < 60) {
      return '剛剛';
    }

    // Minutes ago (< 1 hour)
    if (diffMinutes < 60) {
      return `${diffMinutes}分鐘前`;
    }

    // Hours ago (< 24 hours)
    if (diffHours < 24) {
      return `${diffHours}小時前`;
    }

    // Days ago (< 7 days)
    if (diffDays < 7) {
      if (diffDays === 1) {
        return '昨天';
      }
      return `${diffDays}天前`;
    }

    // Weeks ago (< 30 days)
    if (diffDays < 30) {
      const weeks = Math.floor(diffDays / 7);
      return `${weeks}週前`;
    }

    // Months ago (< 365 days)
    if (diffDays < 365) {
      const months = Math.floor(diffDays / 30);
      return `${months}個月前`;
    }

    // Years ago
    const years = Math.floor(diffDays / 365);
    return `${years}年前`;
  } catch (error) {
    console.error('Failed to format relative time:', error);
    return '--';
  }
}

/**
 * Format ISO 8601 timestamp to absolute time (e.g., "2025-12-03 14:30")
 * @param isoString ISO 8601 timestamp string
 * @returns Formatted absolute time string
 */
export function formatAbsoluteTime(isoString: string): string {
  try {
    const date = new Date(isoString);
    return date.toLocaleString('zh-TW', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
  } catch (error) {
    console.error('Failed to format absolute time:', error);
    return '--';
  }
}

/**
 * Format ISO 8601 timestamp with both relative and absolute time for tooltip
 * @param isoString ISO 8601 timestamp string
 * @returns Object with relative and absolute time strings
 */
export function formatTimeWithTooltip(isoString: string): {
  relative: string;
  absolute: string;
} {
  return {
    relative: formatRelativeTime(isoString),
    absolute: formatAbsoluteTime(isoString),
  };
}

/**
 * Format speed priority value
 * @param priority - Priority value (0 = lowest, higher = more priority, default = 10)
 * @param t - Translation function
 * @returns Formatted priority string
 */
export function formatSpeedPriority(priority: number | undefined, t: (key: string) => string): string {
  // Default to 10 if undefined
  const actualPriority = priority ?? 10;

  // Priority 0 displays as "Lowest Priority" instead of "0"
  if (actualPriority === 0) {
    return t('downloads.priority.lowest');
  }

  return actualPriority.toString();
}

/**
 * Get priority display label with description
 * @param priority - Priority value
 * @param t - Translation function
 * @returns Priority label with optional description
 */
export function getPriorityLabel(priority: number | undefined, t: (key: string) => string): string {
  const actualPriority = priority ?? 10;

  if (actualPriority === 0) {
    return t('downloads.priority.lowest');
  }

  // Provide context for common priority levels
  if (actualPriority >= 20) {
    return `${actualPriority} (${t('downloads.priority.high')})`;
  } else if (actualPriority >= 15) {
    return `${actualPriority} (${t('downloads.priority.aboveDefault')})`;
  } else if (actualPriority === 10) {
    return `${actualPriority} (${t('downloads.priority.default')})`;
  } else if (actualPriority >= 5) {
    return `${actualPriority} (${t('downloads.priority.belowDefault')})`;
  } else {
    return `${actualPriority} (${t('downloads.priority.low')})`;
  }
}
