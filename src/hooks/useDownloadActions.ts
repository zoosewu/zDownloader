// Download commands that also update the store with what the backend returns,
// so the UI reflects the change without waiting for the event.
import { useMemo } from 'react';
import { useSetAtom } from 'jotai';
import { backend, type Task, type TaskId } from '../backend';
import { applyEventAtom, upsertTaskAtom } from '../stores/downloadStore';

export function useDownloadActions() {
  const upsert = useSetAtom(upsertTaskAtom);
  const apply = useSetAtom(applyEventAtom);

  return useMemo(() => {
    const one = async (request: Promise<Task>) => upsert(await request);
    const many = async (request: Promise<Task[]>) => (await request).forEach(upsert);
    const removed = (id: TaskId) => apply({ type: 'removed', data: id });

    return {
      pause: (id: TaskId) => one(backend.downloads.pause(id)),
      resume: (id: TaskId) => one(backend.downloads.resume(id)),
      cancel: (id: TaskId) => one(backend.downloads.cancel(id)),
      retry: (id: TaskId) => one(backend.downloads.retry(id)),
      restart: (id: TaskId) => one(backend.downloads.restart(id)),
      /** `deleteFile`: a finished download's file goes to the recycle bin too. */
      remove: async (id: TaskId, deleteFile = false) => {
        await backend.downloads.remove(id, deleteFile);
        removed(id);
      },
      move: (id: TaskId, before: TaskId | null) => one(backend.downloads.move(id, before)),
      pauseAll: () => many(backend.downloads.pauseAll()),
      resumeAll: () => many(backend.downloads.resumeAll()),
      clearFinished: async () => (await backend.downloads.clearFinished()).forEach(removed),
    };
  }, [upsert, apply]);
}
