// Third-party Web API implementations, bundled into src/gen/vendor.js by
// `pnpm build:module-host-js`. Runs first in every isolate; bootstrap.js builds on it.
import 'core-js/actual/url';
import 'core-js/actual/url-search-params';
import 'core-js/actual/structured-clone';
import 'core-js/actual/atob';
import 'core-js/actual/btoa';
import 'core-js/actual/queue-microtask';
import { DOMParser } from 'linkedom';

globalThis.DOMParser = DOMParser;
