// CAPTCHA state management using Jotai
import { atom } from 'jotai';
import { CaptchaChallenge, AuthCredentials } from '../types/captcha';

// Current active CAPTCHA challenge (null if none)
export const currentCaptchaAtom = atom<CaptchaChallenge | null>(null);

// Whether CAPTCHA dialog is visible
export const showCaptchaDialogAtom = atom<boolean>(false);

// Whether login dialog is visible
export const showLoginDialogAtom = atom<boolean>(false);

// Module ID requiring authentication
export const authModuleIdAtom = atom<string | null>(null);

// Stored credentials per module (in-memory, not persisted)
export const moduleCredentialsAtom = atom<Record<string, AuthCredentials>>({});

// Derived atom: show CAPTCHA dialog when challenge exists
export const captchaDialogVisibleAtom = atom(
  (get) => get(currentCaptchaAtom) !== null && get(showCaptchaDialogAtom)
);
