// Behaviour a desktop app should have and a web page does not.
// - The WebView's own context menu (back, reload, inspect) never shows; text fields and selected
//   text keep it for cut, copy and paste. The app's own menus open from React handlers.
// - Something dropped on the page never navigates away from it: links dropped outside the add
//   dialog open the dialog with them (the dialog handles drops on itself).
import { useEffect } from 'react';
import { backend } from '../backend';
import { linksDropped } from '../utils/links';

const editable = (target: EventTarget | null) =>
  target instanceof HTMLElement && (target.isContentEditable || target.closest('input, textarea') !== null);

export function usePageGuards() {
  useEffect(() => {
    const onContextMenu = (e: MouseEvent) => {
      if (editable(e.target) || !window.getSelection()?.isCollapsed) return;
      e.preventDefault();
    };
    const onDragOver = (e: DragEvent) => e.preventDefault();
    const onDrop = async (e: DragEvent) => {
      if (e.defaultPrevented || !e.dataTransfer) return;
      e.preventDefault();
      const links = await linksDropped(e.dataTransfer);
      if (links.length > 0) await backend.addWindow.open(links.map((url) => ({ url, filename: null, headers: [], pageUrl: null }))).catch(() => {});
    };
    document.addEventListener('contextmenu', onContextMenu);
    document.addEventListener('dragover', onDragOver);
    document.addEventListener('drop', onDrop);
    return () => {
      document.removeEventListener('contextmenu', onContextMenu);
      document.removeEventListener('dragover', onDragOver);
      document.removeEventListener('drop', onDrop);
    };
  }, []);
}
