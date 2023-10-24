import { instantiate } from "./v8_valueserializer_wasm.generated.js";

const { deserialize: deserialize_ } = await instantiate();

// deno-lint-ignore no-explicit-any
export function deserialize(bytes: Uint8Array): any {
  const init = deserialize_(bytes);
  return eval(init);
}
