// Malicious Test Module: infinite_loop.js (T097)
//
// Purpose: Test execution timeout enforcement
// Expected: Long-running scripts should be terminated
//
// Security Test: SC-013 (Resource Limit Enforcement - Execution Time)

/**
 * Malicious Module: Attempts infinite loops and long computations
 *
 * Attempts:
 * 1. Simple infinite loop
 * 2. Recursive function without base case
 * 3. Extremely long computation
 * 4. setTimeout/setInterval abuse
 */
export class InfiniteLoopModule {
  constructor() {
    this.name = 'infinite-loop-test';
  }

  canHandle(url) {
    return true;
  }

  async extractDownloadInfo(url, context) {
    context.log.info('[ATTACK] Starting infinite loop/timeout tests');

    // Attack 1: Simple infinite loop
    try {
      context.log.warn('[ATTACK] Attempting infinite while loop');

      const startTime = Date.now();
      let counter = 0;

      while (true) {
        counter++;

        // Check if loop has been running for more than 10 seconds
        if (Date.now() - startTime > 10000) {
          context.log.error('[ATTACK] SUCCESS - Infinite loop ran for 10+ seconds! No timeout!');
          break;
        }

        // Small yield to prevent complete freeze
        if (counter % 1000000 === 0) {
          await new Promise(resolve => setTimeout(resolve, 0));
        }
      }

      context.log.warn(`[ATTACK] Loop completed ${counter} iterations before timeout`);
    } catch (error) {
      context.log.info(`[EXPECTED] Infinite loop terminated: ${error.message}`);
    }

    // Attack 2: Infinite recursion
    try {
      context.log.warn('[ATTACK] Attempting stack overflow via recursion');

      let depth = 0;
      const infiniteRecursion = () => {
        depth++;
        if (depth % 10000 === 0) {
          context.log.warn(`[ATTACK] Recursion depth: ${depth}`);
        }
        return infiniteRecursion();
      };

      infiniteRecursion();

      context.log.error('[ATTACK] SUCCESS - Infinite recursion completed! No stack limit!');
    } catch (error) {
      context.log.info(`[EXPECTED] Recursion limited: ${error.message}`);
    }

    // Attack 3: CPU-intensive computation
    try {
      context.log.warn('[ATTACK] Attempting extremely long computation');

      const startTime = Date.now();
      let result = 0;

      // Compute for a very long time
      for (let i = 0; i < 10_000_000_000; i++) {
        result += Math.sqrt(i) * Math.sin(i) * Math.cos(i);

        if (i % 100_000_000 === 0) {
          const elapsed = Date.now() - startTime;
          context.log.warn(`[ATTACK] Computation progress: ${i/10_000_000_000*100}%, elapsed: ${elapsed}ms`);

          if (elapsed > 30000) {
            context.log.error('[ATTACK] SUCCESS - Computation ran for 30+ seconds! No CPU limit!');
            break;
          }
        }
      }

      context.log.warn(`[ATTACK] Computation completed: ${result}`);
    } catch (error) {
      context.log.info(`[EXPECTED] Long computation terminated: ${error.message}`);
    }

    // Attack 4: Async infinite loop
    try {
      context.log.warn('[ATTACK] Attempting async infinite loop');

      const startTime = Date.now();
      let iterations = 0;

      while (true) {
        await new Promise(resolve => setTimeout(resolve, 1));
        iterations++;

        if (iterations % 1000 === 0) {
          const elapsed = Date.now() - startTime;
          context.log.warn(`[ATTACK] Async loop: ${iterations} iterations, ${elapsed}ms elapsed`);

          if (elapsed > 15000) {
            context.log.error('[ATTACK] SUCCESS - Async loop ran for 15+ seconds! No timeout!');
            break;
          }
        }
      }
    } catch (error) {
      context.log.info(`[EXPECTED] Async loop terminated: ${error.message}`);
    }

    // Attack 5: Promise chain abuse
    try {
      context.log.warn('[ATTACK] Attempting Promise chain abuse');

      let promise = Promise.resolve(0);

      // Create an extremely long promise chain
      for (let i = 0; i < 1_000_000; i++) {
        promise = promise.then(value => value + 1);
      }

      const result = await promise;
      context.log.error(`[ATTACK] SUCCESS - Long promise chain completed: ${result}! No limit!`);
    } catch (error) {
      context.log.info(`[EXPECTED] Promise chain limited: ${error.message}`);
    }

    // Attack 6: Memory + CPU combo
    try {
      context.log.warn('[ATTACK] Attempting combined memory and CPU exhaustion');

      const arrays = [];
      const startTime = Date.now();

      while (true) {
        // Allocate memory and do computation
        const bigArray = new Array(1_000_000).fill(0).map((_, i) => Math.random() * i);
        arrays.push(bigArray);

        // Intensive computation on the array
        const sum = bigArray.reduce((acc, val) => acc + val, 0);

        if (arrays.length % 10 === 0) {
          const elapsed = Date.now() - startTime;
          const memoryMB = (arrays.length * 1_000_000 * 8) / (1024 * 1024);

          context.log.warn(`[ATTACK] Allocated ${memoryMB}MB, elapsed: ${elapsed}ms`);

          if (elapsed > 20000 || memoryMB > 500) {
            context.log.error('[ATTACK] SUCCESS - Combined attack ran too long! No limits!');
            break;
          }
        }
      }
    } catch (error) {
      context.log.info(`[EXPECTED] Combined attack terminated: ${error.message}`);
    }

    throw new Error('Infinite loop test completed (if you see this, timeout protection may have failed)');
  }
}

export default InfiniteLoopModule;
