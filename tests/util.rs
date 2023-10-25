use v8::ValueSerializerHelper;
use v8_valueserializer::Heap;
use v8_valueserializer::Value;

static ONCE: std::sync::Once = std::sync::Once::new();

fn init_v8() {
  v8::V8::initialize_platform(v8::new_default_platform(0, false).make_shared());
  v8::V8::initialize();
}

#[derive(Debug)]
pub struct JsError {
  #[allow(dead_code)]
  message: String,
}

pub struct Isolate {
  isolate: v8::OwnedIsolate,
  context: v8::Global<v8::Context>,
}

impl Isolate {
  pub fn new() -> Isolate {
    ONCE.call_once(init_v8);
    let mut isolate = v8::Isolate::new(Default::default());
    let mut scope = v8::HandleScope::new(&mut isolate);
    let context = v8::Context::new(&mut scope);
    let context = v8::Global::new(&mut scope, context);
    drop(scope);
    Isolate { isolate, context }
  }

  pub fn eval_and_serialize(&mut self, code: &str) -> Result<Vec<u8>, JsError> {
    let scope = &mut v8::HandleScope::new(&mut self.isolate);
    let context = v8::Local::new(scope, &self.context);
    let scope = &mut v8::ContextScope::new(scope, context);
    let source = v8::String::new_from_utf8(
      scope,
      code.as_bytes(),
      v8::NewStringType::Normal,
    )
    .expect("code is valid utf-8 string");

    let tc = &mut v8::TryCatch::new(scope);

    let Some(script) = v8::Script::compile(tc, source, None) else {
      let message = tc.message().unwrap();
      let message = message.get(tc).to_rust_string_lossy(tc);
      return Err(JsError { message });
    };

    let Some(value) = script.run(tc) else {
      let message = tc.message().unwrap();
      let message = message.get(tc).to_rust_string_lossy(tc);
      return Err(JsError { message });
    };

    let mut serializer =
      v8::ValueSerializer::new(tc, Box::new(ValueSerializerImpl));
    serializer.write_header();
    let Some(true) = serializer.write_value(context, value) else {
      let message = tc.message().unwrap();
      let message = message.get(tc).to_rust_string_lossy(tc);
      return Err(JsError { message });
    };

    Ok(serializer.release())
  }
}

struct ValueSerializerImpl;

impl v8::ValueSerializerImpl for ValueSerializerImpl {
  fn throw_data_clone_error<'s>(
    &mut self,
    scope: &mut v8::HandleScope<'s>,
    message: v8::Local<'s, v8::String>,
  ) {
    let exception: v8::Local<'_, v8::Value> =
      v8::Exception::type_error(scope, message);
    scope.throw_exception(exception);
  }
}

pub struct Assert {
  pub value: Value,
  pub heap: Heap,
}

impl std::fmt::Debug for Assert {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "=== VALUE ===")?;
    writeln!(f, "{:?}", self.value)?;
    writeln!(f)?;
    writeln!(f, "=== HEAP ===")?;
    std::fmt::Debug::fmt(&self.heap, f)?;
    writeln!(f)?;
    Ok(())
  }
}

#[macro_export]
macro_rules! deserialize_test {
  ($name:ident $code:expr) => {
    #[test]
    fn $name() {
      let mut isolate = $crate::util::Isolate::new();
      let code = if $code.starts_with("{") {
        format!("({})", $code)
      } else {
        $code.to_string()
      };
      let bytes = isolate
        .eval_and_serialize(code.as_str())
        .expect("eval_and_serialize failed");
      println!("{:?}", bytes);
      let de = v8_valueserializer::ValueDeserializer::default();
      let (value, heap) = de.read(&bytes).expect("parse_v8 failed");
      let assert = $crate::util::Assert {
        value,
        heap,
      };
      insta::with_settings!({
        description => format!("=== SOURCE ===\n{}", $code),
        omit_expression => true,
      }, {
        insta::assert_debug_snapshot!(stringify!($name), assert);
      })
    }
  };
}

#[macro_export]
macro_rules! display_test {
  ($name:ident $code:expr) => {
    #[test]
    fn $name() {
      let mut isolate = $crate::util::Isolate::new();
      let code = if $code.starts_with("{") {
        format!("({})", $code)
      } else {
        $code.to_string()
      };
      let bytes = isolate
        .eval_and_serialize(code.as_str())
        .expect("eval_and_serialize failed");
      println!("{:?}", bytes);
      let de = v8_valueserializer::ValueDeserializer::default();
      let (value, heap) = de.read(&bytes).expect("parse_v8 failed");
      let serialization = v8_valueserializer::display(&heap, &value, v8_valueserializer::DisplayOptions {
        format: v8_valueserializer::DisplayFormat::Repl,
      });
      insta::with_settings!({
        description => format!("=== SOURCE ===\n{}", $code),
        omit_expression => true,
      }, {
        insta::assert_snapshot!(stringify!($name), serialization);
      })
    }
  };
}
