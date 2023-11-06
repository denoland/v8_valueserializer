use v8::Global;
use v8::Local;
use v8::Value;
use v8::ValueDeserializerHelper;
use v8::ValueSerializerHelper;
use v8_valueserializer::Heap;

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

impl Default for Isolate {
  fn default() -> Isolate {
    ONCE.call_once(init_v8);
    let mut isolate = v8::Isolate::new(Default::default());
    let mut scope = v8::HandleScope::new(&mut isolate);
    let context = v8::Context::new(&mut scope);
    let context = v8::Global::new(&mut scope, context);
    drop(scope);
    Isolate { isolate, context }
  }
}

impl Isolate {
  #[allow(dead_code)]
  pub fn deserialize(
    &mut self,
    serialized: &[u8],
  ) -> Result<Global<Value>, JsError> {
    let scope = &mut v8::HandleScope::new(&mut self.isolate);
    let context = v8::Local::new(scope, &self.context);
    let scope = &mut v8::ContextScope::new(scope, context);

    let tc = &mut v8::TryCatch::new(scope);
    let mut deserializer = v8::ValueDeserializer::new(
      tc,
      Box::new(ValueDeserializerImpl),
      serialized,
    );

    let Some(true) = deserializer.read_header(context) else {
      let message = tc.message().unwrap();
      let message = message.get(tc).to_rust_string_lossy(tc);
      return Err(JsError { message });
    };

    let Some(value) = deserializer.read_value(context) else {
      let message = tc.message().unwrap();
      let message = message.get(tc).to_rust_string_lossy(tc);
      return Err(JsError { message });
    };

    let global = Global::new(tc, value);

    Ok(global)
  }

  #[allow(dead_code)]
  pub fn eval(&mut self, code: &str) -> Result<Global<Value>, JsError> {
    let scope = &mut v8::HandleScope::new(&mut self.isolate);
    let context = v8::Local::new(scope, &self.context);
    let scope = &mut v8::ContextScope::new(scope, context);

    let tc = &mut v8::TryCatch::new(scope);

    let Some(source) =
      v8::String::new_from_utf8(tc, code.as_bytes(), v8::NewStringType::Normal)
    else {
      let message = tc.message().unwrap();
      let message = message.get(tc).to_rust_string_lossy(tc);
      return Err(JsError { message });
    };

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

    let global = Global::new(tc, value);
    Ok(global)
  }

  #[allow(dead_code)]
  pub fn serialize_value(
    &mut self,
    value: Global<Value>,
  ) -> Result<Vec<u8>, JsError> {
    let scope = &mut v8::HandleScope::new(&mut self.isolate);
    let context = v8::Local::new(scope, &self.context);
    let scope = &mut v8::ContextScope::new(scope, context);

    let tc = &mut v8::TryCatch::new(scope);
    let mut serializer =
      v8::ValueSerializer::new(tc, Box::new(ValueSerializerImpl));

    let value = Local::new(tc, &value);

    serializer.write_header();
    let Some(true) = serializer.write_value(context, value) else {
      let message = tc.message().unwrap();
      let message = message.get(tc).to_rust_string_lossy(tc);
      return Err(JsError { message });
    };

    let buf = serializer.release();
    Ok(buf)
  }
}

struct ValueDeserializerImpl;

impl v8::ValueDeserializerImpl for ValueDeserializerImpl {
  fn read_host_object<'s>(
    &mut self,
    scope: &mut v8::HandleScope<'s>,
    _value_deserializer: &mut dyn v8::ValueDeserializerHelper,
  ) -> Option<v8::Local<'s, v8::Object>> {
    let msg = v8::String::new(
      scope,
      "Deno deserializer: read_host_object not implemented",
    )
    .unwrap();
    let exc = v8::Exception::error(scope, msg);
    scope.throw_exception(exc);
    None
  }

  fn get_shared_array_buffer_from_id<'s>(
    &mut self,
    scope: &mut v8::HandleScope<'s>,
    _transfer_id: u32,
  ) -> Option<v8::Local<'s, v8::SharedArrayBuffer>> {
    let msg = v8::String::new(
      scope,
      "Deno deserializer: get_shared_array_buffer_from_id not implemented",
    )
    .unwrap();
    let exc = v8::Exception::error(scope, msg);
    scope.throw_exception(exc);
    None
  }

  fn get_wasm_module_from_id<'s>(
    &mut self,
    scope: &mut v8::HandleScope<'s>,
    _clone_id: u32,
  ) -> Option<v8::Local<'s, v8::WasmModuleObject>> {
    let msg = v8::String::new(
      scope,
      "Deno deserializer: get_wasm_module_from_id not implemented",
    )
    .unwrap();
    let exc = v8::Exception::error(scope, msg);
    scope.throw_exception(exc);
    None
  }
}

struct ValueSerializerImpl;

impl v8::ValueSerializerImpl for ValueSerializerImpl {
  fn throw_data_clone_error<'s>(
    &mut self,
    scope: &mut v8::HandleScope<'s>,
    message: v8::Local<'s, v8::String>,
  ) {
    let exc = v8::Exception::error(scope, message);
    scope.throw_exception(exc);
  }
}

pub struct Assert {
  pub value: v8_valueserializer::Value,
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
macro_rules! serde_test {
  ($name:ident $code:expr) => {
    #[test]
    fn $name() {
      let mut isolate = $crate::util::Isolate::default();
      let code = if $code.starts_with("{") {
        format!("({})", $code)
      } else {
        $code.to_string()
      };

      let input_js_value = isolate.eval(code.as_str()).expect("eval failed");
      let input_bytes = isolate
        .serialize_value(input_js_value)
        .expect("serialize_value failed");
      println!("injs_bytes {:?}", input_bytes);
      let de = v8_valueserializer::ValueDeserializer::default();

      let (value, heap) = de.read(&input_bytes).expect("original parse_v8 failed");
      let assert = $crate::util::Assert {
        value,
        heap,
      };
      insta::with_settings!({
        description => format!("=== SOURCE ===\n{}", $code),
        omit_expression => true,
      }, {
        insta::assert_debug_snapshot!(concat!("de_", stringify!($name)), assert);
      });

      let ser = v8_valueserializer::ValueSerializer::default();
      let rs_ser_bytes = ser.finish(&assert.heap, &assert.value).expect("serialize failed");
      println!("rsserbytes {:?}", rs_ser_bytes);

      let rs_roundtripped_value = isolate
        .deserialize(&rs_ser_bytes)
        .expect("rs deserialize failed");
      let rs_roundtripped_bytes = isolate
        .serialize_value(rs_roundtripped_value)
        .expect("rs serialize_value failed");
      println!("rsrt_bytes {:?}", rs_roundtripped_bytes);

      let de = v8_valueserializer::ValueDeserializer::default();
      let (rs_rt_value, rs_rt_heap) = de.read(&rs_roundtripped_bytes).expect("parse_v8 failed");

      if !v8_valueserializer::value_eq((&assert.value, &assert.heap), (&rs_rt_value, &rs_rt_heap)) {
        println!("=== EXPECTED ===");
        println!("{:?}", assert);

        let rt_assert = $crate::util::Assert {
          value: rs_rt_value,
          heap: rs_rt_heap,
        };
        println!("=== ROUNDTRIPPED ===");
        println!("{:?}", rt_assert);

        panic!("serialized roundtrip failed");
      }


      let display = v8_valueserializer::display(&assert.heap, &assert.value, v8_valueserializer::DisplayOptions {
        format: v8_valueserializer::DisplayFormat::Repl,
      });
      insta::with_settings!({
        description => format!("=== SOURCE ===\n{}", $code),
        omit_expression => true,
      }, {
        insta::assert_snapshot!(concat!("display_", stringify!($name)), display);
      });

      let eval = v8_valueserializer::display(&assert.heap, &assert.value, v8_valueserializer::DisplayOptions {
        format: v8_valueserializer::DisplayFormat::Eval,
      });
      let display_rt_js_value = isolate.eval(eval.as_str()).expect("eval failed");
      let display_rt_bytes = isolate
        .serialize_value(display_rt_js_value)
        .expect("serialize_value failed");
      println!("display_rt_bytes {:?}", display_rt_bytes);

      let de = v8_valueserializer::ValueDeserializer::default();
      let (display_rt_value, display_rt_heap) = de.read(&display_rt_bytes).expect("parse_v8 failed");

      if !v8_valueserializer::value_eq((&assert.value, &assert.heap), (&display_rt_value, &display_rt_heap)) {
        println!("=== EXPECTED ===");
        println!("{:?}", assert);

        let rt_assert = $crate::util::Assert {
          value: display_rt_value,
          heap: display_rt_heap,
        };
        println!("=== ROUNDTRIPPED ===");
        println!("{:?}", rt_assert);

        panic!("display roundtrip failed");
      }

    }
  };
}

#[macro_export]
macro_rules! display_test {
  ($name:ident $code:expr) => {
    #[test]
    fn $name() {
      let mut isolate = $crate::util::Isolate::default();
      let code = if $code.starts_with("{") {
        format!("({})", $code)
      } else {
        $code.to_string()
      };
      let eval_value = isolate.eval(code.as_str()).expect("eval failed");
      let eval_bytes = isolate
        .serialize_value(eval_value)
        .expect("serialize_value failed");
      println!("eval_bytes {:?}", eval_bytes);
      let de = v8_valueserializer::ValueDeserializer::default();
      let (value, heap) = de.read(&eval_bytes).expect("parse_v8 failed");
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
