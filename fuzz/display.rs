use afl::fuzz;
use v8_valueserializer::display;
use v8_valueserializer::value_eq;
use v8_valueserializer::DisplayOptions;
use v8_valueserializer::ParseError;
use v8_valueserializer::ParseErrorKind;
use v8_valueserializer::ValueDeserializer;
mod util;

fn main() {
  fuzz!(|data: &[u8]| {
    let mut isolate = util::Isolate::new();

    let original_bytes = match isolate.deserialize(data) {
      Ok(value) => match isolate.serialize_value(value) {
        Ok(bytes) => bytes,
        Err(err) => {
          println!("v8 serialize failed: {:?}", err);
          return;
        }
      },
      Err(err) => {
        println!("v8 deserialize failed: {:?}", err);
        return;
      }
    };

    let deserializer = ValueDeserializer::default();
    let res = deserializer.read(&original_bytes);
    let (original_value, original_heap) = match res {
      Ok(val) => val,
      Err(ParseError {
        kind:
          ParseErrorKind::InvalidWireFormatVersion(..)
          | ParseErrorKind::HostObjectNotSupported
          | ParseErrorKind::SharedObjectNotSupported
          | ParseErrorKind::SharedArrayBufferNotSupported
          | ParseErrorKind::WasmModuleTransferNotSupported
          | ParseErrorKind::InvalidRegExpFlags(..) // https://bugs.chromium.org/p/v8/issues/detail?id=14412
          | ParseErrorKind::TooDeeplyNested,
        ..
      }) => {
        println!("parse bad");
        return;
      }
      Err(e) => {
        println!("parse_v8 failed: {:?}", e);
        panic!("");
      }
    };

    let code = display(
      &original_heap,
      &original_value,
      DisplayOptions {
        format: v8_valueserializer::DisplayFormat::Expression,
      },
    );
    println!("==== code ========");
    println!("{}", code);
    println!("==== code end ====");

    let roundtripped_bytes = match isolate.eval(&code) {
      Ok(value) => match isolate.serialize_value(value) {
        Ok(bytes) => bytes,
        Err(err) => {
          println!("v8 serialize failed: {:?}", err);
          panic!("");
        }
      },
      Err(err) => {
        println!("eval of code failed: {:?}", err);
        panic!("");
      }
    };
    let deserializer = ValueDeserializer::default();
    let (roundtripped_value, roundtripped_heap) =
      match deserializer.read(&roundtripped_bytes) {
        Ok(val) => val,
        Err(e) => {
          println!("==== original bytes ========");
          println!("{:?}", original_bytes);
          println!("==== original bytes end ====");
          println!("==== roundtripped bytes ========");
          println!("{:?}", roundtripped_bytes);
          println!("==== roundtripped bytes end ====");
          println!("roundtripped parse_v8 failed: {:?}", e);
          panic!("");
        }
      };

    if !value_eq(
      (&roundtripped_value, &roundtripped_heap),
      (&original_value, &original_heap),
    ) {
      println!("roundtrip failed");
      println!("==== roundtripped value ========");
      println!("{:#?}", roundtripped_heap);
      println!("{:#?}", roundtripped_value);
      println!("==== roundtripped value end ====");
      panic!("");
    }
  });
}
