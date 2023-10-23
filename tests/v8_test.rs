use crate::util::Isolate;
use v8_valueserializer::display;
mod util;

#[test]
fn test_serialize() {
  let mut isolate = Isolate::new();
  let data = isolate
    .eval_and_serialize(
      r#"
    const b = { };
    const x = new Set([{ b }, "foo", 1, 2]);
    b.x = x;
    x
  "#,
    )
    .unwrap();
  let de = v8_valueserializer::ValueDeserializer::default();
  let (value, heap) = de.read(&data).unwrap();
  let new_code = display(&heap, &value);
  println!("{}", new_code);
}
