// Runs before the module, once per isolate.
// deno_core's console prints to stdout, which carries the protocol: replace it with one that
// writes to stderr. Then hide Deno.core so module code reaches the host only through `zoo`.
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

  const zoo = Object.freeze({
    log: Object.freeze({ ...log }),
    fail(message, { hint } = {}) {
      const error = new Error(message);
      if (hint !== undefined) error.hint = String(hint);
      return error;
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
  delete globalThis.Deno;
})(Deno.core);
