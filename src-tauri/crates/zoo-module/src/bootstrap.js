// Runs before the module, once per isolate.
// deno_core's console prints to stdout, which carries the protocol: replace it with one that
// writes to stderr. Then hide deno_core's op table so module code reaches the host only through
// the APIs in web.js and `zoo`.
((core) => {
  const print = core.print;
  const format = (args) =>
    args.map((arg) => {
      if (typeof arg === "string") return arg;
      try {
        return JSON.stringify(arg) ?? String(arg);
      } catch {
        return String(arg);
      }
    }).join(" ");
  const log = {};
  for (const level of ["debug", "info", "warn", "error"]) {
    log[level] = (...args) => print(`[${level}] ${format(args)}\n`, true);
  }
  globalThis.console = { ...log, log: log.info };

  const { op_zoo_credential, op_zoo_wait, op_zoo_captcha } = core.ops;
  const fail = (message, { hint } = {}) => {
    const error = new Error(message);
    if (hint !== undefined) error.hint = String(hint);
    return error;
  };
  const solved = (solution) => {
    if (solution === null) throw fail("captcha was not solved", { hint: "the user declined or did not answer in time" });
    return solution;
  };

  const zoo = Object.freeze({
    log: Object.freeze({ ...log }),
    fail,
    credentials: Object.freeze({
      has: (key) => op_zoo_credential(String(key)) !== null,
      // An opaque stand-in for the value: put it in a fetch url, header or body and the host
      // sends the real value. Module code never sees credential values.
      placeholder(key) {
        const placeholder = op_zoo_credential(String(key));
        if (placeholder === null) {
          throw fail(`credential '${key}' is not available`, {
            hint: "declare it in manifest.json permissions.credentials; check zoo.credentials.has(key) first",
          });
        }
        return placeholder;
      },
    }),
    captcha: Object.freeze({
      recaptcha: async ({ siteKey, pageUrl }) =>
        solved(await op_zoo_captcha({ kind: "recaptcha", siteKey: String(siteKey), pageUrl: String(pageUrl) })),
      image: async (imageUrl) => solved(await op_zoo_captcha({ kind: "image", url: String(imageUrl) })),
    }),
    // A site countdown. Shown to the user; does not count against the time limit.
    wait: async (seconds, reason) => {
      const ms = Math.round(Number(seconds) * 1000);
      if (!(ms >= 0)) throw new TypeError("zoo.wait: seconds must be a non-negative number");
      await op_zoo_wait(ms, reason === undefined || reason === null ? "" : String(reason));
    },
  });

  // Called by the generated entry module with the module's default export.
  // Settles with a JSON string: {"ok": result} | {"err": {message, hint, stack}} | {"invalid": why}.
  const run = async (module, url) => {
    if (module === null || typeof module !== "object" || typeof module.resolve !== "function") {
      return JSON.stringify({ invalid: "the default export has no resolve function" });
    }
    try {
      const result = await module.resolve(url, zoo);
      return JSON.stringify({ ok: result ?? null });
    } catch (e) {
      const isObject = e !== null && typeof e === "object";
      return JSON.stringify({
        err: {
          message: isObject && "message" in e ? String(e.message) : String(e),
          hint: isObject && typeof e.hint === "string" ? e.hint : null,
          stack: isObject && typeof e.stack === "string" ? e.stack : null,
        },
      });
    }
  };
  Object.defineProperty(globalThis, "__zooRun", { value: run });
  // deno_core exposes its op table both as Deno.core and as __bootstrap.core.
  delete globalThis.Deno;
  delete globalThis.__bootstrap;
  // core-js keeps its state in closures; the global is only for sharing between copies.
  delete globalThis["__core-js_shared__"];
})(Deno.core);
