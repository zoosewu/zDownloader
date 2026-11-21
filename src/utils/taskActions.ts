// What can be done to a download in each state (mirrors the transitions zoo-core allows).
import type { TaskState } from '../backend';

/** `deleteFile`: remove a finished download and move its file to the recycle bin (asks first). */
export type Action = 'pause' | 'resume' | 'cancel' | 'retry' | 'openFile' | 'openFolder' | 'remove' | 'deleteFile';

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
      return ['openFile', 'openFolder', 'remove', 'deleteFile'];
  }
}

/** Start over from the first byte: anything not running and not already finished. */
export const canRestart = (state: TaskState) => state !== 'running' && state !== 'completed';
