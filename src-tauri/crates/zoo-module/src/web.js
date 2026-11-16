// Web platform APIs for site modules, on top of vendor.js (URL, URLSearchParams,
// structuredClone, atob/btoa, queueMicrotask, DOMParser).
// Behaves like the browser where modules can tell; network I/O goes through the host's Policy.
// Not provided: streaming bodies (response.body), Blob/File, WebSocket, Workers.
((core) => {
  const { op_zoo_fetch, op_zoo_random, op_zoo_digest } = core.ops;

  // ── DOMException, Event, EventTarget (just enough for AbortSignal) ──

  class DOMException extends Error {
    constructor(message = "", name = "Error") {
      super(message);
      this.name = name;
    }
  }

  class Event {
    constructor(type) {
      this.type = String(type);
      this.target = null;
      this.defaultPrevented = false;
    }
    preventDefault() {
      this.defaultPrevented = true;
    }
  }

  const listenersOf = new WeakMap();
  class EventTarget {
    addEventListener(type, listener, options) {
      if (typeof listener !== "function" && typeof listener?.handleEvent !== "function") return;
      if (!listenersOf.has(this)) listenersOf.set(this, []);
      const once = typeof options === "object" && options !== null && !!options.once;
      listenersOf.get(this).push({ type: String(type), listener, once });
    }
    removeEventListener(type, listener) {
      const list = listenersOf.get(this);
      if (!list) return;
      const i = list.findIndex((l) => l.type === String(type) && l.listener === listener);
      if (i >= 0) list.splice(i, 1);
    }
    dispatchEvent(event) {
      event.target = this;
      const handler = this[`on${event.type}`];
      if (typeof handler === "function") handler.call(this, event);
      for (const entry of [...(listenersOf.get(this) ?? [])]) {
        if (entry.type !== event.type) continue;
        if (entry.once) this.removeEventListener(entry.type, entry.listener);
        if (typeof entry.listener === "function") entry.listener.call(this, event);
        else entry.listener.handleEvent(event);
      }
      return !event.defaultPrevented;
    }
  }

  // ── Timers ──

  // deno_core timers are objects; scripts get numeric ids like in browsers.
  const timers = new Map();
  const timer = (repeat) => (callback, delay = 0, ...args) => {
    if (typeof callback !== "function") throw new TypeError("timer callback must be a function");
    const ms = Math.max(0, Number(delay) || 0);
    let id;
    const fire = () => {
      if (!repeat) timers.delete(id);
      callback(...args);
    };
    const t = core.createTimer(fire, ms, undefined, repeat, true, false);
    id = t._timerId;
    timers.set(id, t);
    return id;
  };
  const clearTimer = (id) => {
    const t = timers.get(id);
    if (t === undefined) return;
    timers.delete(id);
    core.cancelTimer(t);
  };
  const unrefTimer = (id) => {
    const t = timers.get(id);
    if (t !== undefined) core.unrefTimer(t);
  };

  // ── AbortController / AbortSignal ──

  class AbortSignal extends EventTarget {
    #aborted = false;
    #reason = undefined;
    constructor(key) {
      super();
      if (key !== signalKey) throw new TypeError("Illegal constructor; use new AbortController()");
      this.onabort = null;
    }
    get aborted() {
      return this.#aborted;
    }
    get reason() {
      return this.#reason;
    }
    throwIfAborted() {
      if (this.#aborted) throw this.#reason;
    }
    static abort(reason) {
      const signal = new AbortSignal(signalKey);
      abortSignal(signal, reason);
      return signal;
    }
    static timeout(ms) {
      const signal = new AbortSignal(signalKey);
      const id = timer(false)(() => abortSignal(signal, new DOMException("The operation timed out.", "TimeoutError")), ms);
      unrefTimer(id); // like browsers: a pending timeout signal does not keep the call alive
      return signal;
    }
    static any(signals) {
      const signal = new AbortSignal(signalKey);
      for (const source of signals) {
        if (source.aborted) {
          abortSignal(signal, source.reason);
          break;
        }
        source.addEventListener("abort", () => abortSignal(signal, source.reason), { once: true });
      }
      return signal;
    }
    static _abort(signal, reason) {
      if (signal.#aborted) return;
      signal.#aborted = true;
      signal.#reason = reason === undefined ? new DOMException("This operation was aborted", "AbortError") : reason;
      signal.dispatchEvent(new Event("abort"));
    }
  }
  const signalKey = Symbol("AbortSignal");
  const abortSignal = AbortSignal._abort;
  delete AbortSignal._abort;

  class AbortController {
    #signal = new AbortSignal(signalKey);
    get signal() {
      return this.#signal;
    }
    abort(reason) {
      abortSignal(this.#signal, reason);
    }
  }

  // ── Encoding ──

  const toBytes = (input) => {
    if (input instanceof Uint8Array) return input;
    if (ArrayBuffer.isView(input)) return new Uint8Array(input.buffer, input.byteOffset, input.byteLength);
    if (input instanceof ArrayBuffer) return new Uint8Array(input);
    throw new TypeError("expected an ArrayBuffer or ArrayBufferView");
  };

  class TextEncoder {
    get encoding() {
      return "utf-8";
    }
    encode(input = "") {
      return core.encode(String(input));
    }
    encodeInto(input, dest) {
      const bytes = core.encode(String(input));
      const written = Math.min(bytes.length, dest.length);
      dest.set(bytes.subarray(0, written));
      return { read: String(input).length, written };
    }
  }

  const countReplacements = (text) => text.split("�").length - 1;
  const countEncodedReplacements = (bytes) => {
    let n = 0;
    for (let i = 0; i + 2 < bytes.length; i++) {
      if (bytes[i] === 0xef && bytes[i + 1] === 0xbf && bytes[i + 2] === 0xbd) n++;
    }
    return n;
  };

  class TextDecoder {
    #fatal;
    constructor(label = "utf-8", options = {}) {
      const normalized = String(label).trim().toLowerCase();
      if (!["utf-8", "utf8", "unicode-1-1-utf-8"].includes(normalized)) {
        throw new RangeError(`TextDecoder: unsupported encoding '${label}' (only utf-8)`);
      }
      this.#fatal = !!options.fatal;
    }
    get encoding() {
      return "utf-8";
    }
    get fatal() {
      return this.#fatal;
    }
    decode(input) {
      if (input === undefined) return "";
      const bytes = toBytes(input);
      const text = core.decode(bytes);
      // Invalid input decodes to U+FFFD; a genuine U+FFFD is the bytes EF BF BD.
      if (this.#fatal && countReplacements(text) !== countEncodedReplacements(bytes)) {
        throw new TypeError("TextDecoder: invalid utf-8");
      }
      return text;
    }
  }

  // ── Headers ──

  const normalizeName = (name) => {
    const n = String(name).toLowerCase();
    if (!/^[!#$%&'*+\-.^_`|~0-9a-z]+$/.test(n)) throw new TypeError(`invalid header name: ${name}`);
    return n;
  };
  const normalizeValue = (value) => String(value).replace(/^[\t\n\r ]+|[\t\n\r ]+$/g, "");

  class Headers {
    #list = []; // [lowercased name, value], in insertion order
    constructor(init) {
      if (init instanceof Headers) {
        for (const [name, value] of init.#list) this.#list.push([name, value]);
      } else if (Array.isArray(init) || (init && typeof init[Symbol.iterator] === "function")) {
        for (const pair of init) {
          const [name, value] = [...pair];
          this.append(name, value);
        }
      } else if (init && typeof init === "object") {
        for (const name of Object.keys(init)) this.append(name, init[name]);
      }
    }
    append(name, value) {
      this.#list.push([normalizeName(name), normalizeValue(value)]);
    }
    set(name, value) {
      const n = normalizeName(name);
      this.#list = this.#list.filter(([k]) => k !== n);
      this.#list.push([n, normalizeValue(value)]);
    }
    delete(name) {
      const n = normalizeName(name);
      this.#list = this.#list.filter(([k]) => k !== n);
    }
    get(name) {
      const n = normalizeName(name);
      const values = this.#list.filter(([k]) => k === n).map(([, v]) => v);
      return values.length === 0 ? null : values.join(", ");
    }
    has(name) {
      const n = normalizeName(name);
      return this.#list.some(([k]) => k === n);
    }
    getSetCookie() {
      return this.#list.filter(([k]) => k === "set-cookie").map(([, v]) => v);
    }
    *entries() {
      const names = [...new Set(this.#list.map(([k]) => k))].sort();
      for (const name of names) {
        if (name === "set-cookie") for (const v of this.getSetCookie()) yield [name, v];
        else yield [name, this.get(name)];
      }
    }
    *keys() {
      for (const [name] of this.entries()) yield name;
    }
    *values() {
      for (const [, value] of this.entries()) yield value;
    }
    forEach(callback, thisArg) {
      for (const [name, value] of this.entries()) callback.call(thisArg, value, name, this);
    }
    [Symbol.iterator]() {
      return this.entries();
    }
    static _raw(headers) {
      return headers.#list.map(([k, v]) => [k, v]);
    }
  }
  const rawHeaders = Headers._raw;
  delete Headers._raw;

  // ── FormData (string fields only) ──

  class FormData {
    #entries = [];
    append(name, value) {
      if (typeof value === "object" && value !== null) throw new TypeError("FormData: only string values are supported");
      this.#entries.push([String(name), String(value)]);
    }
    set(name, value) {
      this.delete(name);
      this.append(name, value);
    }
    get(name) {
      return this.#entries.find(([k]) => k === String(name))?.[1] ?? null;
    }
    getAll(name) {
      return this.#entries.filter(([k]) => k === String(name)).map(([, v]) => v);
    }
    has(name) {
      return this.#entries.some(([k]) => k === String(name));
    }
    delete(name) {
      this.#entries = this.#entries.filter(([k]) => k !== String(name));
    }
    *entries() {
      yield* this.#entries.map(([k, v]) => [k, v]);
    }
    [Symbol.iterator]() {
      return this.entries();
    }
  }

  const randomHex = (bytes) =>
    Array.from(crypto.getRandomValues(new Uint8Array(bytes)), (b) => b.toString(16).padStart(2, "0")).join("");

  /** [bytes | null, content type | null] for a fetch/Response body. */
  const extractBody = (body) => {
    if (body === undefined || body === null) return [null, null];
    if (typeof body === "string") return [core.encode(body), "text/plain;charset=UTF-8"];
    if (body instanceof URLSearchParams) {
      return [core.encode(body.toString()), "application/x-www-form-urlencoded;charset=UTF-8"];
    }
    if (body instanceof FormData) {
      const boundary = `----zoo${randomHex(12)}`;
      const escape = (s) => s.replace(/"/g, "%22").replace(/\r?\n|\r/g, "\r\n");
      let text = "";
      for (const [name, value] of body) {
        text += `--${boundary}\r\nContent-Disposition: form-data; name="${escape(name)}"\r\n\r\n${value}\r\n`;
      }
      text += `--${boundary}--\r\n`;
      return [core.encode(text), `multipart/form-data; boundary=${boundary}`];
    }
    if (ArrayBuffer.isView(body) || body instanceof ArrayBuffer) return [toBytes(body).slice(), null];
    return [core.encode(String(body)), "text/plain;charset=UTF-8"];
  };

  // ── Response ──

  class Response {
    #bytes;
    #used = false;
    constructor(body = null, init = {}) {
      const [bytes, contentType] = extractBody(body);
      this.#bytes = bytes ?? new Uint8Array(0);
      this.status = init.status ?? 200;
      this.statusText = init.statusText ?? "";
      this.headers = new Headers(init.headers);
      if (contentType && !this.headers.has("content-type")) this.headers.set("content-type", contentType);
      this.url = init._url ?? "";
      this.redirected = init._redirected ?? false;
      this.type = init._url ? "basic" : "default";
    }
    get ok() {
      return this.status >= 200 && this.status < 300;
    }
    get bodyUsed() {
      return this.#used;
    }
    get body() {
      throw new TypeError("response.body (streams) is not supported; use text(), json(), arrayBuffer() or bytes()");
    }
    #consume() {
      if (this.#used) return Promise.reject(new TypeError("Body has already been consumed"));
      this.#used = true;
      return Promise.resolve(this.#bytes);
    }
    bytes() {
      return this.#consume().then((b) => b.slice());
    }
    arrayBuffer() {
      return this.#consume().then((b) => b.slice().buffer);
    }
    text() {
      return this.#consume().then((b) => core.decode(b));
    }
    json() {
      return this.text().then(JSON.parse);
    }
    clone() {
      if (this.#used) throw new TypeError("Body has already been consumed");
      return new Response(this.#bytes.slice(), {
        status: this.status,
        statusText: this.statusText,
        headers: this.headers,
        _url: this.url,
        _redirected: this.redirected,
      });
    }
    static json(data, init = {}) {
      const headers = new Headers(init.headers);
      if (!headers.has("content-type")) headers.set("content-type", "application/json");
      return new Response(JSON.stringify(data), { ...init, headers });
    }
    static error() {
      return new Response(null, { status: 0 });
    }
    static redirect(url, status = 302) {
      return new Response(null, { status, headers: { location: String(url) } });
    }
  }

  // ── fetch ──

  const METHODS = ["DELETE", "GET", "HEAD", "OPTIONS", "POST", "PUT", "PATCH"];

  async function fetch(input, init = {}) {
    const url = new URL(typeof input === "object" && input !== null && "url" in input ? input.url : String(input));
    const method = String(init.method ?? input?.method ?? "GET").toUpperCase();
    if (!METHODS.includes(method)) throw new TypeError(`fetch: unsupported method ${method}`);
    const headers = new Headers(init.headers ?? input?.headers);
    const [body, contentType] = extractBody(init.body);
    if (body && (method === "GET" || method === "HEAD")) throw new TypeError(`fetch: ${method} cannot have a body`);
    if (contentType && !headers.has("content-type")) headers.set("content-type", contentType);
    const redirect = init.redirect ?? "follow";
    if (!["follow", "manual", "error"].includes(redirect)) throw new TypeError(`fetch: bad redirect mode ${redirect}`);

    const signal = init.signal ?? null;
    signal?.throwIfAborted();
    const request = op_zoo_fetch({ url: url.href, method, headers: rawHeaders(headers), body, redirect });
    const raw = await (signal
      ? new Promise((resolve, reject) => {
        signal.addEventListener("abort", () => reject(signal.reason), { once: true });
        request.then(resolve, reject);
      })
      : request);
    return new Response(raw.body, {
      status: raw.status,
      statusText: raw.statusText,
      headers: raw.headers,
      _url: raw.url,
      _redirected: raw.redirected,
    });
  }

  // ── crypto ──

  const DIGESTS = { "SHA-1": "sha1", "SHA-256": "sha256", "SHA-384": "sha384", "SHA-512": "sha512" };
  const crypto = {
    getRandomValues(array) {
      if (!ArrayBuffer.isView(array) || array instanceof Float32Array || array instanceof Float64Array) {
        throw new TypeError("getRandomValues expects an integer TypedArray");
      }
      if (array.byteLength > 65536) throw new DOMException("at most 65536 bytes", "QuotaExceededError");
      op_zoo_random(toBytes(array));
      return array;
    },
    randomUUID() {
      const b = crypto.getRandomValues(new Uint8Array(16));
      b[6] = (b[6] & 0x0f) | 0x40;
      b[8] = (b[8] & 0x3f) | 0x80;
      const h = Array.from(b, (x) => x.toString(16).padStart(2, "0")).join("");
      return `${h.slice(0, 8)}-${h.slice(8, 12)}-${h.slice(12, 16)}-${h.slice(16, 20)}-${h.slice(20)}`;
    },
    subtle: {
      async digest(algorithm, data) {
        const name = String(typeof algorithm === "object" ? algorithm.name : algorithm).toUpperCase();
        if (!DIGESTS[name]) throw new DOMException(`digest: unsupported algorithm ${name}`, "NotSupportedError");
        return op_zoo_digest(DIGESTS[name], toBytes(data)).buffer;
      },
    },
  };

  const timeOrigin = Date.now();

  Object.assign(globalThis, {
    DOMException,
    Event,
    EventTarget,
    AbortController,
    AbortSignal,
    TextEncoder,
    TextDecoder,
    Headers,
    FormData,
    Response,
    fetch,
    crypto,
    setTimeout: timer(false),
    setInterval: timer(true),
    clearTimeout: clearTimer,
    clearInterval: clearTimer,
    performance: { timeOrigin, now: () => Date.now() - timeOrigin },
    self: globalThis,
  });
})(Deno.core);
