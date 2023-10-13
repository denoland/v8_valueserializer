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
  let (value, heap) = v8_valueserializer::parse_v8(&data).unwrap();
  let new_code = display(&heap, &value);
  println!("{}", new_code);
}
