import { send, statusKey, takeoverEnabled } from './src/core.js';

const $ = (id) => document.getElementById(id);
const t = (key) => chrome.i18n.getMessage(key);

$('takeover-label').textContent = t('takeover');
// The one limit worth knowing before turning it on: downloads made by a form POST.
$('takeover-row').title = t('takeoverLimit');
$('status').textContent = t('statusChecking');

const toggle = $('takeover');
toggle.checked = await takeoverEnabled(chrome);
toggle.addEventListener('change', () => chrome.storage.local.set({ takeover: toggle.checked }));

const answer = await send(chrome, { type: 'ping' });
$('dot').className = `dot ${answer.ok ? 'ok' : 'bad'}`;
$('status').textContent = t(statusKey(answer));
