import {
  DisplayFormat,
  instantiate,
} from "./v8_valueserializer_wasm.generated.js";

const { display: display_ } = await instantiate();

// deno-lint-ignore no-explicit-any
export function deserialize(bytes: Uint8Array): any {
  const init = display_(bytes, DisplayFormat.Eval);
  return eval(init);
}

export function display(
  bytes: Uint8Array,
  format: "repl" | "eval" | "expression" = "repl",
) {
  const format_ = format === "repl"
    ? DisplayFormat.Repl
    : format === "eval"
    ? DisplayFormat.Eval
    : format === "expression"
    ? DisplayFormat.Expression
    : (() => {
      throw new Error("Invalid format: " + format);
    })();
  return display_(bytes, format_);
}
