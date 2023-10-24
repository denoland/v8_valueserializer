use v8::ValueDeserializerHelper;

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

  pub fn parse_serialized(&mut self, serialized: &[u8]) -> Result<(), JsError> {
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

    let Some(_value) = deserializer.read_value(context) else {
      let message = tc.message().unwrap();
      let message = message.get(tc).to_rust_string_lossy(tc);
      return Err(JsError { message });
    };

    Ok(())
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
