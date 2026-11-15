// Service worker: wires browser events to src/core.js. Listeners are registered at the top
// level so the browser can wake the worker for them.
import { createMenu, createTakeover, onMenuClick } from './src/core.js';

chrome.runtime.onInstalled.addListener(() => createMenu(chrome));
chrome.contextMenus.onClicked.addListener((info, tab) => onMenuClick(chrome, info, tab, navigator.userAgent));
chrome.downloads.onDeterminingFilename.addListener(createTakeover(chrome, { userAgent: navigator.userAgent }));
