// What can be done to a download in each state (mirrors the transitions zoo-core allows).
import type { TaskState } from '../backend';

export type Action = 'pause' | 'resume' | 'cancel' | 'retry' | 'remove';

/** The row's buttons, in order. */
export function actionsFor(state: TaskState): Action[] {
  switch (state) {
    case 'running':
    case 'queued':
    case 'scheduled':
      return ['pause', 'cancel', 'remove'];
    case 'paused':
      return ['resume', 'cancel', 'remove'];
    case 'failed':
      return ['retry', 'cancel', 'remove'];
    case 'cancelled':
      return ['retry', 'remove'];
    case 'completed':
      return ['remove'];
  }
}

/** Start over from the first byte: anything not running and not already finished. */
export const canRestart = (state: TaskState) => state !== 'running' && state !== 'completed';
