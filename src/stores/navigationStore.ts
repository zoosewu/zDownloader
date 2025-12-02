// Navigation state management using Jotai
import { atom } from 'jotai';

export type PageType = 'main' | 'settings' | 'add-download';

// Current page atom
export const currentPageAtom = atom<PageType>('main');

// Previous page for back navigation
export const previousPageAtom = atom<PageType | null>(null);

// Navigation parameters (e.g., pre-filled URL)
export const navigationParamsAtom = atom<Record<string, any>>({});
