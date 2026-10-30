// CAPTCHA state management using Jotai
import { atom } from 'jotai';
import { CaptchaChallenge } from '../types/captcha';

// Current active CAPTCHA challenge (null if none)
export const currentCaptchaAtom = atom<CaptchaChallenge | null>(null);

// Whether CAPTCHA dialog is visible
export const showCaptchaDialogAtom = atom<boolean>(false);

// Derived atom: show CAPTCHA dialog when challenge exists
export const captchaDialogVisibleAtom = atom(
  (get) => get(currentCaptchaAtom) !== null && get(showCaptchaDialogAtom)
);
