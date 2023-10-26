#!/usr/bin/env -S deno run -A
import { $, PathRef } from "https://deno.land/x/dax@0.35.0/mod.ts";

Deno.addSignalListener("SIGINT", () => {
  Deno.writeTextFileSync("foo", "bar");
  main.kill("SIGINT");
  // @ts-ignore whoop
  workers.forEach((worker) => worker.kill("SIGINT"));
});

const VALID_TARGETS = ["deserializer", "display"];
const target = Deno.args[0];
if (!VALID_TARGETS.includes(target)) {
  console.error(`invalid target: ${target}`);
  Deno.exit(1);
}

await $`cargo afl build --examples --release`;

const path = new PathRef(new URL(import.meta.url));

const main = $`cargo afl fuzz -M 00 -i ${path.join("..", target, "in")} -o ${
  path.join("..", target, "out")
} ${path.join("..", "..", "target", "release", "examples", `fuzz_${target}`)}`
  .spawn();

const workerCount = Math.floor(navigator.hardwareConcurrency - 3);
console.log(workerCount);

// deno-lint-ignore no-explicit-any
const workers: Promise<any>[] = [];
for (let i = 1; i <= workerCount; i++) {
  await new Promise((resolve) => setTimeout(resolve, 1000));
  workers.push(
    $`cargo afl fuzz -S ${i.toString().padStart(2, "0")} -i ${
      path.join("..", target, "in")
    } -o ${path.join("..", target, "out")} ${
      path.join("..", "..", "target", "release", "examples", `fuzz_${target}`)
    }`.spawn().catch((e) => {
      console.error(e);
    }),
  );
}

await Promise.allSettled([main, ...workers]);
await $`killall -9 ${`fuzz_${target}`}`;
