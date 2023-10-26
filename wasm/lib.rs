use v8_valueserializer::DisplayOptions;
use v8_valueserializer::ValueDeserializer;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(u8)]
pub enum DisplayFormat {
  Repl = 0,
  Expression = 1,
  Eval = 2,
}

#[wasm_bindgen]
pub fn display(
  bytes: Vec<u8>,
  format: DisplayFormat,
) -> Result<String, JsError> {
  let deserializer = ValueDeserializer::default();
  let (value, heap) = deserializer.read(&bytes)?;
  let str = v8_valueserializer::display(
    &heap,
    &value,
    DisplayOptions {
      format: match format {
        DisplayFormat::Repl => v8_valueserializer::DisplayFormat::Repl,
        DisplayFormat::Expression => {
          v8_valueserializer::DisplayFormat::Expression
        }
        DisplayFormat::Eval => v8_valueserializer::DisplayFormat::Eval,
      },
    },
  );
  Ok(str)
}
