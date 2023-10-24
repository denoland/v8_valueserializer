use v8_valueserializer::display;
use v8_valueserializer::DisplayFormat;
use v8_valueserializer::DisplayOptions;
use v8_valueserializer::ValueDeserializer;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn deserialize(bytes: Vec<u8>) -> Result<String, JsError> {
  let deserializer = ValueDeserializer::default();
  let (value, heap) = deserializer.read(&bytes)?;
  let str = display(
    &heap,
    &value,
    DisplayOptions {
      format: DisplayFormat::Eval,
    },
  );
  Ok(str)
}
