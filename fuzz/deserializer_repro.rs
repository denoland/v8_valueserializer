use v8_valueserializer::ParseError;
use v8_valueserializer::ParseErrorKind;
use v8_valueserializer::ValueDeserializer;
mod util;

fn main() {
  let path = std::env::args().nth(1).unwrap();
  let mut crash_paths = std::fs::read_dir(path)
    .unwrap()
    .map(|x| x.unwrap().path())
    .collect::<Vec<_>>();
  crash_paths.sort();
  for crash_path in crash_paths {
    println!("\n===========================================================");
    println!("crash_path: {:?}", crash_path);
    let data = std::fs::read(crash_path).unwrap();
    let deserializer = ValueDeserializer::default();
    let res = deserializer.read(&data);
    // If there is a parse error, check whether V8 can parse this data. If V8
    // can not parse the input, the input is not valid and we can skip parsing.
    if res.is_err() {
      let mut isolate = util::Isolate::new();
      if let Err(err) = isolate.parse_serialized(&data) {
        println!("v8 failed: {:?}", err);
        continue;
      }
    }
    match res {
      Ok(_)
      | Err(ParseError {
        kind:
          ParseErrorKind::InvalidWireFormatVersion(..)
          | ParseErrorKind::HostObjectNotSupported
          | ParseErrorKind::SharedObjectNotSupported
          | ParseErrorKind::SharedArrayBufferNotSupported
          | ParseErrorKind::WasmModuleTransferNotSupported,
        ..
      }) => {
        println!("ok");
      }
      Err(e) => {
        println!("!!! parse_v8 failed: {:?}", e);
      }
    }
  }
}
