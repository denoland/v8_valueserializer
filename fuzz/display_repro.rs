use std::path::PathBuf;

use v8_valueserializer::display;
use v8_valueserializer::value_eq;
use v8_valueserializer::DisplayOptions;
use v8_valueserializer::ParseError;
use v8_valueserializer::ParseErrorKind;
use v8_valueserializer::ValueDeserializer;
mod util;

fn main() {
  let path = PathBuf::from(std::env::args().nth(1).unwrap());
  let crash_paths = if path.is_dir() {
    let fuzzer_paths = std::fs::read_dir(path)
      .unwrap()
      .map(|x| x.unwrap().path())
      .collect::<Vec<_>>();
    let mut crash_paths = fuzzer_paths
      .into_iter()
      .flat_map(|path| {
        std::fs::read_dir(path.join("crashes"))
          .unwrap()
          .map(|x| x.unwrap().path())
          .filter(|path| !path.ends_with("README.txt"))
      })
      .collect::<Vec<_>>();
    crash_paths.sort();
    crash_paths
  } else {
    vec![path]
  };
  for crash_path in crash_paths {
    println!("\n===========================================================");
    println!("crash_path: {:?}", crash_path);
    let data = std::fs::read(crash_path).unwrap();
    let mut isolate = util::Isolate::new();

    let original_bytes = match isolate.deserialize(&data) {
      Ok(value) => match isolate.serialize_value(value) {
        Ok(bytes) => bytes,
        Err(err) => {
          println!("v8 serialize failed: {:?}", err);
          continue;
        }
      },
      Err(err) => {
        println!("v8 deserialize failed: {:?}", err);
        continue;
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
        continue;
      }
      Err(e) => {
        println!("!!! parse_v8 failed: {:?}", e);
        continue;
      }
    };

    println!("==== original value ========");
    println!("{:#?}", original_heap);
    println!("{:#?}", original_value);
    println!("==== original value end ====\n");

    let code = display(
      &original_heap,
      &original_value,
      DisplayOptions {
        format: v8_valueserializer::DisplayFormat::Expression,
      },
    );
    println!("==== code ========");
    println!("{}", code);
    println!("==== code end ====\n");

    let roundtripped_bytes = match isolate.eval(&code) {
      Ok(value) => match isolate.serialize_value(value) {
        Ok(bytes) => bytes,
        Err(err) => {
          println!("!!! v8 serialize failed: {:?}", err);
          continue;
        }
      },
      Err(err) => {
        println!("!!! eval of code failed: {:?}", err);
        continue;
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
          continue;
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
      continue;
    }
    println!("ok");
  }
}
