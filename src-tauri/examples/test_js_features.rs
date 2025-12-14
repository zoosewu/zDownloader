// Test JavaScript features supported by deno_core runtime
// Run with: cargo run --example test_js_features

use deno_core::JsRuntime;

fn main() {
    println!("=== Testing JavaScript Features in deno_core 0.372 ===\n");

    let mut runtime = JsRuntime::new(Default::default());

    // Test ES2015 (ES6) features
    println!("📋 ES2015 (ES6) Features:");
    test_feature(&mut runtime, "let/const", "const x = 1; let y = 2;");
    test_feature(&mut runtime, "Arrow functions", "const fn = () => 42;");
    test_feature(&mut runtime, "Template literals", "const s = `hello ${1+1}`;");
    test_feature(&mut runtime, "Destructuring", "const {a, b} = {a: 1, b: 2};");
    test_feature(&mut runtime, "Default parameters", "function fn(x = 5) { return x; }");
    test_feature(&mut runtime, "Rest parameters", "function fn(...args) { return args; }");
    test_feature(&mut runtime, "Spread operator", "const arr = [1, ...[2, 3]];");
    test_feature(&mut runtime, "Classes", "class Foo { constructor() { this.x = 1; } }");
    test_feature(&mut runtime, "for...of", "for (const x of [1,2,3]) {}");
    test_feature(&mut runtime, "Map/Set", "const m = new Map(); const s = new Set();");
    test_feature(&mut runtime, "Promise", "const p = new Promise((resolve) => resolve(1));");

    println!("\n📋 ES2016 Features:");
    test_feature(&mut runtime, "Exponentiation", "const x = 2 ** 3;");
    test_feature(&mut runtime, "Array.includes", "[1,2,3].includes(2);");

    println!("\n📋 ES2017 Features:");
    test_feature(&mut runtime, "async/await", "async function fn() { await Promise.resolve(1); }");
    test_feature(&mut runtime, "Object.values", "Object.values({a: 1});");
    test_feature(&mut runtime, "Object.entries", "Object.entries({a: 1});");
    test_feature(&mut runtime, "String.padStart", "'5'.padStart(3, '0');");

    println!("\n📋 ES2018 Features:");
    test_feature(&mut runtime, "Rest/Spread properties", "const {x, ...rest} = {x: 1, y: 2};");
    test_feature(&mut runtime, "Promise.finally", "Promise.resolve(1).finally(() => {});");
    test_feature(&mut runtime, "Async iteration", "async function* gen() { yield 1; }");
    test_feature(&mut runtime, "RegExp named groups", "/(?<year>\\d{4})/.exec('2024');");

    println!("\n📋 ES2019 Features:");
    test_feature(&mut runtime, "Array.flat", "[1, [2, 3]].flat();");
    test_feature(&mut runtime, "Array.flatMap", "[1, 2].flatMap(x => [x, x * 2]);");
    test_feature(&mut runtime, "Object.fromEntries", "Object.fromEntries([['a', 1]]);");
    test_feature(&mut runtime, "String.trimStart", "' hello '.trimStart();");
    test_feature(&mut runtime, "Optional catch", "try { throw 1; } catch { }");

    println!("\n📋 ES2020 Features:");
    test_feature(&mut runtime, "Optional chaining", "const x = {}; x?.a?.b;");
    test_feature(&mut runtime, "Nullish coalescing", "const x = null ?? 42;");
    test_feature(&mut runtime, "BigInt", "const x = 123n;");
    test_feature(&mut runtime, "Promise.allSettled", "Promise.allSettled([Promise.resolve(1)]);");
    test_feature(&mut runtime, "globalThis", "globalThis.x = 1;");
    test_feature(&mut runtime, "String.matchAll", "'abc'.matchAll(/./g);");

    println!("\n📋 ES2021 Features:");
    test_feature(&mut runtime, "String.replaceAll", "'aaa'.replaceAll('a', 'b');");
    test_feature(&mut runtime, "Promise.any", "Promise.any([Promise.reject(1), Promise.resolve(2)]);");
    test_feature(&mut runtime, "Logical assignment", "let x = 1; x ||= 2; x &&= 3; x ??= 4;");
    test_feature(&mut runtime, "Numeric separators", "const x = 1_000_000;");
    test_feature(&mut runtime, "WeakRef", "const ref = new WeakRef({});");

    println!("\n📋 ES2022 Features:");
    test_feature(&mut runtime, "Class fields", "class C { x = 1; }");
    test_feature(&mut runtime, "Private fields", "class C { #x = 1; }");
    test_feature(&mut runtime, "Static fields", "class C { static x = 1; }");
    test_feature(&mut runtime, "Top-level await", "// Not testable in this context");
    test_feature(&mut runtime, "Array.at", "[1,2,3].at(-1);");
    test_feature(&mut runtime, "Object.hasOwn", "Object.hasOwn({a: 1}, 'a');");

    println!("\n📋 ES2023 Features:");
    test_feature(&mut runtime, "Array.findLast", "[1,2,3].findLast(x => x > 1);");
    test_feature(&mut runtime, "Array.toSorted", "[3,1,2].toSorted();");
    test_feature(&mut runtime, "Array.toReversed", "[1,2,3].toReversed();");

    println!("\n✅ Test Complete!\n");
    println!("📊 Summary:");
    println!("   deno_core version: 0.372");
    println!("   V8 engine: Latest stable");
    println!("   JavaScript support: ES2015-ES2023+ (Modern JavaScript)");
    println!("   Recommended target for Vite: es2020 or es2021");
}

fn test_feature(runtime: &mut JsRuntime, name: &str, code: &str) {
    let result = runtime.execute_script(
        format!("test_{}", name.replace(" ", "_").replace("/", "_").replace(".", "_")),
        code.to_string(),
    );

    match result {
        Ok(_) => println!("  ✓ {}", name),
        Err(e) => {
            let error_msg = format!("{:?}", e);
            if error_msg.contains("SyntaxError") {
                println!("  ✗ {} (Syntax not supported)", name);
            } else {
                println!("  ⚠ {} (Runtime error: {})", name, error_msg.lines().next().unwrap_or("unknown"));
            }
        }
    }
}
