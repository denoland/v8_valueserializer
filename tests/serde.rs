mod util;

// special values
serde_test!(undefined r#"undefined"#);
serde_test!(null r#"null"#);

// booleans
serde_test!(bool_true r#"true"#);
serde_test!(bool_false r#"false"#);

// small integers
serde_test!(i32 r#"1"#);
serde_test!(i32_neg r#"1000"#);
serde_test!(i32_zero r#"0"#);
serde_test!(i32_max r#"2147483647"#);
serde_test!(i32_min r#"-2147483648"#);

// doubles
serde_test!(double r#"1.1"#);
serde_test!(double_neg r#"-1.1"#);
serde_test!(double_neg_zero r#"-0"#);
serde_test!(double_nan r#"NaN"#);
serde_test!(double_infinity r#"Infinity"#);
serde_test!(double_neg_infinity r#"-Infinity"#);
serde_test!(double_max r#"1.7976931348623157e+308"#);
serde_test!(double_min r#"5e-324"#);

// bigints
serde_test!(bigint r#"1n"#);
serde_test!(bigint_neg r#"-1n"#);
serde_test!(bigint_zero r#"0n"#);
serde_test!(bigint_u64_max r#"18446744073709551615n"#);
serde_test!(bigint_u128_max r#"340282366920938463463374607431768211455n"#);

// strings
serde_test!(string_empty r#"''"#);
serde_test!(string_one_byte r#"'asd'"#);
serde_test!(string_two_byte r#"'asd 🌎'"#);
serde_test!(string_0_byte r#""è÷\u{0}""#);
serde_test!(string_unpaired_surrogate r#""foo\ud800bar""#);
serde_test!(string_escape_double_quote r#"'"'"#);
serde_test!(string_escape_backslash r#""\\""#);

// boolean primitive wrapper object
serde_test!(boolean_primitive_true r#"new Boolean(true)"#);
serde_test!(boolean_primitive_false r#"new Boolean(false)"#);

// number primitive wrapper object
serde_test!(number_primitive r#"new Number(1)"#);
serde_test!(number_primitive_neg r#"new Number(-1)"#);
serde_test!(number_primitive_zero r#"new Number(0)"#);
serde_test!(number_primitive_nan r#"new Number(NaN)"#);
serde_test!(number_primitive_infinity r#"new Number(Infinity)"#);
serde_test!(number_primitive_infinity_neg r#"new Number(-Infinity)"#);
serde_test!(number_primitive_max r#"new Number(1.7976931348623157e+308)"#);
serde_test!(number_primitive_min r#"new Number(5e-324)"#);

// bigint primitive wrapper object
serde_test!(bigint_primitive r#"Object(1n)"#);
serde_test!(bigint_primitive_neg r#"Object(-1n)"#);
serde_test!(bigint_primitive_zero r#"Object(0n)"#);
serde_test!(bigint_primitive_u64_max r#"Object(18446744073709551615n)"#);
serde_test!(bigint_primitive_u128_max r#"Object(340282366920938463463374607431768211455n)"#);

// string primitive wrapper object
serde_test!(string_primitive_empty r#"new String('')"#);
serde_test!(string_primitive_one_byte r#"new String('asd')"#);
serde_test!(string_primitive_two_byte r#"new String('asd 🌎')"#);

// regexp
serde_test!(regexp r#"/asd/gi"#);
serde_test!(regexp_empty r#"new RegExp("(?:)")"#);
serde_test!(regexp_two_byte r#"/🗄️/"#);

// date
serde_test!(date r#"new Date(1)"#);
serde_test!(date_zero r#"new Date(0)"#);
serde_test!(date_max r#"new Date(8640000000000000)"#);
serde_test!(date_min r#"new Date(-8640000000000000)"#);
serde_test!(date_invalid r#"new Date(NaN)"#);

// object
serde_test!(object_empty r#"{}"#);
serde_test!(object_one_property r#"{"a": 1}"#);
serde_test!(object_two_properties r#"{"a": 1, "b": 2}"#);
serde_test!(object_smi_property_key r#"{[1]: 3}"#);
serde_test!(object_smi_property_key_neg r#"{[-1]: 4}"#);
serde_test!(object_smi_property_key_as_str r#"{"1": 5}"#);
serde_test!(object_two_byte_property_key r#"{"foo🗄️": 6}"#);
serde_test!(object_nested r#"{"a": {"b": true}}"#);

// sparse array
serde_test!(sparse_array_empty r#"new Array(0)"#);
serde_test!(sparse_array_empty_length_one r#"new Array(1)"#);
serde_test!(sparse_array_one_element_length_one r#"
const arr = new Array(1);
arr[0] = 1;
arr
"#);
serde_test!(sparse_array_one_element_length_two r#"
const arr = new Array(2);
arr[1] = 1;
arr
"#);
serde_test!(sparse_array_one_element_length_two_with_properties r#"
const arr = new Array();
arr[1] = 1;
arr.foo = "bar";
arr
"#);
serde_test!(sparse_array_literal_with_hole r#"[1, 2, /* hole */, 4]"#);
serde_test!(sparse_array_with_object r#"[/* hole */, {a: 2}]"#);
serde_test!(sparse_array_circular_self r#"const arr = new Array(2); arr[0] = arr; arr"#);
serde_test!(sparse_array_with_properties_ordered r#"
const c = { };
const b = { c };
const a = { b };
const arr = new Array(3);
arr[2] = b; 
arr.x = 1;
arr.c = c;
c.arr = arr;
arr.foo = "bar";
c
"#);

// dense array
serde_test!(dense_array r#"[1, 2]"#);
serde_test!(dense_array_empty r#"[]"#);
serde_test!(dense_array_one_element r#"[1]"#);
serde_test!(dense_array_two_elements_multi_type r#"[1, "asd"]"#);
serde_test!(dense_array_with_properties r#"
const arr = ["asd", 1];
arr.foo = "bar";
arr
"#);
serde_test!(dense_array_with_properties_ordered r#"
const c = { };
const b = { c };
const a = { b };
const arr = ["asd", 1, b];
arr.x = 1;
arr.c = c;
c.arr = arr;
arr.foo = "bar";
c
"#);
serde_test!(dense_array_with_object r#"[1, { a: true }]"#);
serde_test!(dense_array_circular_self r#"const arr = [undefined]; arr[0] = arr; arr"#);

// map
serde_test!(map_empty r#"new Map()"#);
serde_test!(map_one_element r#"new Map([["a", 1]])"#);
serde_test!(map_two_elements r#"new Map([["a", 1], ["b", 2]])"#);
serde_test!(map_object_key r#"new Map([[{ a: true }, 1]])"#);
serde_test!(map_object_value r#"new Map([[1, { b: true }]])"#);
serde_test!(map_object_key_and_value r#"new Map([[{ a: true }, { b: true }]])"#);
serde_test!(map_circular_self r#"const m = new Map(); m.set(1, m); m"#);
serde_test!(map_circular_multi r#"const a = { m: new Map() }; a.m.set(1, a); a"#);

// set
serde_test!(set_empty r#"new Set()"#);
serde_test!(set_one_element r#"new Set([1])"#);
serde_test!(set_two_elements r#"new Set([1, 2])"#);
serde_test!(set_object_element r#"new Set([{ a: true }])"#);
serde_test!(set_circular_self r#"const s = new Set(); s.add(s); s"#);
serde_test!(set_circular_multi r#"const a = { s: new Set() }; a.s.add(a); a"#);

// arraybuffer
serde_test!(arraybuffer_empty r#"new ArrayBuffer(0)"#);
serde_test!(arraybuffer_one_byte r#"new ArrayBuffer(1)"#);
serde_test!(arraybuffer_with_data r#"new Uint8Array([1,2]).buffer"#);

// resizable arraybuffer
serde_test!(resizable_arraybuffer_empty r#"new ArrayBuffer(2, { maxByteLength: 10 })"#);

// uint8array
serde_test!(uint8array_empty r#"new Uint8Array()"#);
serde_test!(uint8array_zeroed r#"new Uint8Array(8)"#);
serde_test!(uint8array_zeroed2 r#"new Uint8Array([0, 0, 0])"#);
serde_test!(uint8array_one_byte r#"new Uint8Array([1])"#);
serde_test!(uint8array_two_bytes r#"new Uint8Array([1, 2])"#);
serde_test!(uint8array_many_bytes r#"new Uint8Array([0, 1, 2, 3, 4, 5, 6, 7, 8])"#);
serde_test!(uint8array_subarray r#"new Uint8Array([1, 2]).subarray(1, 2)"#);
serde_test!(uint8array_resizable r#"new Uint8Array(new ArrayBuffer(2, { maxByteLength: 10 }))"#);
serde_test!(uint8array_resizable_non_tracking r#"new Uint8Array(new ArrayBuffer(2, { maxByteLength: 10 })).subarray(0, 1)"#);
serde_test!(uint8array_resizable_with_data r#"const a = new Uint8Array(new ArrayBuffer(2, { maxByteLength: 10 })); a.set([1, 2]); a"#);

// uint8clampedarray
serde_test!(uint8clampedarray_empty r#"new Uint8ClampedArray()"#);
serde_test!(uint8clampedarray_zeroed r#"new Uint8ClampedArray(8)"#);
serde_test!(uint8clampedarray_zeroed2 r#"new Uint8ClampedArray([0, 0, 0])"#);
serde_test!(uint8clampedarray_one_byte r#"new Uint8ClampedArray([1])"#);
serde_test!(uint8clampedarray_two_bytes r#"new Uint8ClampedArray([1, 2])"#);
serde_test!(uint8clampedarray_many_bytes r#"new Uint8ClampedArray([0, 1, 2, 3, 4, 5, 6, 7, 8])"#);
serde_test!(uint8clampedarray_subarray r#"new Uint8ClampedArray([1, 2]).subarray(1, 2)"#);
serde_test!(uint8clampedarray_resizable r#"new Uint8ClampedArray(new ArrayBuffer(2, { maxByteLength: 10 }))"#);
serde_test!(uint8clampedarray_resizable_non_tracking r#"new Uint8ClampedArray(new ArrayBuffer(2, { maxByteLength: 10 })).subarray(0, 1)"#);
serde_test!(uint8clampedarray_resizable_with_data r#"const a = new Uint8ClampedArray(new ArrayBuffer(2, { maxByteLength: 10 })); a.set([1, 2]); a"#);

// int8array
serde_test!(int8array_empty r#"new Int8Array()"#);
serde_test!(int8array_zeroed r#"new Int8Array(8)"#);
serde_test!(int8array_zeroed2 r#"new Int8Array([0, 0, 0])"#);
serde_test!(int8array_one_byte r#"new Int8Array([-1])"#);
serde_test!(int8array_two_bytes r#"new Int8Array([1, -2])"#);
serde_test!(int8array_many_bytes r#"new Int8Array([0, -1, 2, -3, 4, -5, 6, -7, 8])"#);
serde_test!(int8array_subarray r#"new Int8Array([1, 2]).subarray(1, 2)"#);
serde_test!(int8array_resizable r#"new Int8Array(new ArrayBuffer(2, { maxByteLength: 10 }))"#);
serde_test!(int8array_resizable_non_tracking r#"new Int8Array(new ArrayBuffer(2, { maxByteLength: 10 })).subarray(0, 1)"#);
serde_test!(int8array_resizable_with_data r#"const a = new Int8Array(new ArrayBuffer(2, { maxByteLength: 10 })); a.set([1, -2]); a"#);

// uint16array
serde_test!(uint16array_empty r#"new Uint16Array()"#);
serde_test!(uint16array_zeroed r#"new Uint16Array(4)"#);
serde_test!(uint16array_zeroed2 r#"new Uint16Array([0, 0, 0])"#);
serde_test!(uint16array_one_byte r#"new Uint16Array([1])"#);
serde_test!(uint16array_two_bytes r#"new Uint16Array([1, 2])"#);
serde_test!(uint16array_many_bytes r#"new Uint16Array([0, 1, 2, 3, 4, 5, 6, 7, 8])"#);
serde_test!(uint16array_subarray r#"new Uint16Array([1, 2]).subarray(1, 2)"#);
serde_test!(uint16array_resizable r#"new Uint16Array(new ArrayBuffer(2, { maxByteLength: 10 }))"#);
serde_test!(uint16array_resizable_non_tracking r#"new Uint16Array(new ArrayBuffer(4, { maxByteLength: 10 })).subarray(0, 1)"#);
serde_test!(uint16array_resizable_with_data r#"const a = new Uint16Array(new ArrayBuffer(4, { maxByteLength: 10 })); a.set([1, 2]); a"#);

// int16array
serde_test!(int16array_empty r#"new Int16Array()"#);
serde_test!(int16array_zeroed r#"new Int16Array(4)"#);
serde_test!(int16array_zeroed2 r#"new Int16Array([0, 0, 0])"#);
serde_test!(int16array_one_byte r#"new Int16Array([-1])"#);
serde_test!(int16array_two_bytes r#"new Int16Array([1, -2])"#);
serde_test!(int16array_many_bytes r#"new Int16Array([0, -1, 2, -3, 4, -5, 6, -7, 8])"#);
serde_test!(int16array_subarray r#"new Int16Array([1, 2]).subarray(1, 2)"#);
serde_test!(int16array_resizable r#"new Int16Array(new ArrayBuffer(2, { maxByteLength: 10 }))"#);
serde_test!(int16array_resizable_non_tracking r#"new Uint16Array(new ArrayBuffer(4, { maxByteLength: 10 })).subarray(0, 1)"#);
serde_test!(int16array_resizable_with_data r#"const a = new Int16Array(new ArrayBuffer(4, { maxByteLength: 10 })); a.set([1, -2]); a"#);

// uint32array
serde_test!(uint32array_empty r#"new Uint32Array()"#);
serde_test!(uint32array_zeroed r#"new Uint32Array(2)"#);
serde_test!(uint32array_zeroed2 r#"new Uint32Array([0, 0, 0])"#);
serde_test!(uint32array_one_byte r#"new Uint32Array([1])"#);
serde_test!(uint32array_two_bytes r#"new Uint32Array([1, 2])"#);
serde_test!(uint32array_many_bytes r#"new Uint32Array([0, 1, 2, 3, 4, 5, 6, 7, 8])"#);
serde_test!(uint32array_subarray r#"new Uint32Array([1, 2]).subarray(1, 2)"#);
serde_test!(uint32array_resizable r#"new Uint32Array(new ArrayBuffer(4, { maxByteLength: 12 }))"#);
serde_test!(uint32array_resizable_non_tracking r#"new Uint32Array(new ArrayBuffer(8, { maxByteLength: 12 })).subarray(0, 1)"#);
serde_test!(uint32array_resizable_with_data r#"const a = new Uint32Array(new ArrayBuffer(8, { maxByteLength: 12 })); a.set([1, 2]); a"#);

// int32array
serde_test!(int32array_empty r#"new Int32Array()"#);
serde_test!(int32array_zeroed r#"new Int32Array(2)"#);
serde_test!(int32array_zeroed2 r#"new Int32Array([0, 0, 0])"#);
serde_test!(int32array_one_byte r#"new Int32Array([-1])"#);
serde_test!(int32array_two_bytes r#"new Int32Array([1, -2])"#);
serde_test!(int32array_many_bytes r#"new Int32Array([0, -1, 2, -3, 4, -5, 6, -7, 8])"#);
serde_test!(int32array_subarray r#"new Int32Array([1, 2]).subarray(1, 2)"#);
serde_test!(int32array_resizable r#"new Int32Array(new ArrayBuffer(4, { maxByteLength: 12 }))"#);
serde_test!(int32array_resizable_non_tracking r#"new Int32Array(new ArrayBuffer(8, { maxByteLength: 12 })).subarray(0, 1)"#);
serde_test!(int32array_resizable_with_data r#"const a = new Int32Array(new ArrayBuffer(8, { maxByteLength: 12 })); a.set([1, -2]); a"#);

// biguint64array
serde_test!(biguint64array_empty r#"new BigUint64Array()"#);
serde_test!(biguint64array_zeroed r#"new BigUint64Array(1)"#);
serde_test!(biguint64array_zeroed2 r#"new BigUint64Array([0n, 0n, 0n])"#);
serde_test!(biguint64array_one_byte r#"new BigUint64Array([1n])"#);
serde_test!(biguint64array_two_bytes r#"new BigUint64Array([1n, 2n])"#);
serde_test!(biguint64array_many_bytes r#"new BigUint64Array([0n, 1n, 2n, 3n, 4n, 5n, 6n, 7n, 8n])"#);
serde_test!(biguint64array_subarray r#"new BigUint64Array([1n, 2n]).subarray(1, 2)"#);
serde_test!(biguint64array_resizable r#"new BigUint64Array(new ArrayBuffer(8, { maxByteLength: 16 }))"#);
serde_test!(biguint64array_resizable_non_tracking r#"new BigUint64Array(new ArrayBuffer(16, { maxByteLength: 24 })).subarray(0, 1)"#);
serde_test!(biguint64array_resizable_with_data r#"const a = new BigUint64Array(new ArrayBuffer(16, { maxByteLength: 24 })); a.set([1n, 2n]); a"#);

// bigint64array
serde_test!(bigint64array_empty r#"new BigInt64Array()"#);
serde_test!(bigint64array_zeroed r#"new BigInt64Array(1)"#);
serde_test!(bigint64array_zeroed2 r#"new BigInt64Array([0n, 0n, 0n])"#);
serde_test!(bigint64array_one_byte r#"new BigInt64Array([-1n])"#);
serde_test!(bigint64array_two_bytes r#"new BigInt64Array([1n, -2n])"#);
serde_test!(bigint64array_many_bytes r#"new BigInt64Array([0n, -1n, 2n, -3n, 4n, -5n, 6n, -7n, 8n])"#);
serde_test!(bigint64array_subarray r#"new BigInt64Array([1n, 2n]).subarray(1, 2)"#);
serde_test!(bigint64array_resizable r#"new BigInt64Array(new ArrayBuffer(8, { maxByteLength: 16 }))"#);
serde_test!(bigint64array_resizable_non_tracking r#"new BigInt64Array(new ArrayBuffer(16, { maxByteLength: 24 })).subarray(0, 1)"#);
serde_test!(bigint64array_resizable_with_data r#"const a = new BigInt64Array(new ArrayBuffer(16, { maxByteLength: 24 })); a.set([1n, -2n]); a"#);

// float32array
serde_test!(float32array_empty r#"new Float32Array()"#);
serde_test!(float32array_zeroed r#"new Float32Array(2)"#);
serde_test!(float32array_zeroed2 r#"new Float32Array([0, 0, 0])"#);
serde_test!(float32array_one_byte r#"new Float32Array([1.0])"#);
serde_test!(float32array_two_bytes r#"new Float32Array([1.0, -2.5])"#);
serde_test!(float32array_many_bytes r#"new Float32Array([0, -1.5, 2, -3.5, NaN, -5.5, -Infinity, -0, Infinity])"#);
serde_test!(float32array_subarray r#"new Float32Array([1, 2]).subarray(1, 2)"#);
serde_test!(float32array_resizable r#"new Float32Array(new ArrayBuffer(4, { maxByteLength: 12 }))"#);
serde_test!(float32array_resizable_non_tracking r#"new Float32Array(new ArrayBuffer(8, { maxByteLength: 12 })).subarray(0, 1)"#);
serde_test!(float32array_resizable_with_data r#"const a = new Float32Array(new ArrayBuffer(8, { maxByteLength: 12 })); a.set([1.0, -2.5]); a"#);

// float64array
serde_test!(float64array_empty r#"new Float64Array()"#);
serde_test!(float64array_zeroed r#"new Float64Array(1)"#);
serde_test!(float64array_zeroed2 r#"new Float64Array([0, 0, 0])"#);
serde_test!(float64array_one_byte r#"new Float64Array([1.0])"#);
serde_test!(float64array_two_bytes r#"new Float64Array([1.0, -2.5])"#);
serde_test!(float64array_many_bytes r#"new Float64Array([0, -1.5, 2, -3.5, NaN, -5.5, -Infinity, -0, Infinity])"#);
serde_test!(float64array_subarray r#"new Float64Array([1, 2]).subarray(1, 2)"#);
serde_test!(float64array_resizable r#"new Float64Array(new ArrayBuffer(8, { maxByteLength: 16 }))"#);
serde_test!(float64array_resizable_non_tracking r#"new Float64Array(new ArrayBuffer(16, { maxByteLength: 24 })).subarray(0, 1)"#);
serde_test!(float64array_resizable_with_data r#"const a = new Float64Array(new ArrayBuffer(16, { maxByteLength: 24 })); a.set([1.0, -2.5]); a"#);

// dataview
serde_test!(dataview_empty r#"new DataView(new ArrayBuffer(0))"#);
serde_test!(dataview_zeroed r#"new DataView(new ArrayBuffer(8))"#);
serde_test!(dataview_zeroed2 r#"new DataView(new Uint8Array([0, 0, 0]).buffer)"#);
serde_test!(dataview_one_byte r#"new DataView(new Uint8Array([1]).buffer)"#);
serde_test!(dataview_two_bytes r#"new DataView(new Uint8Array([1, 2]).buffer)"#);
serde_test!(dataview_many_bytes r#"new DataView(new Uint8Array([0, 1, 2, 3, 4, 5, 6, 7, 8]).buffer)"#);
serde_test!(dataview_subarray r#"new DataView(new Uint8Array([1, 2]).buffer, 1, 1)"#);
serde_test!(dataview_resizable r#"new DataView(new ArrayBuffer(8, { maxByteLength: 16 }))"#);
serde_test!(dataview_resizable_non_tracking r#"new DataView(new ArrayBuffer(2, { maxByteLength: 10 }), 0, 1)"#);
serde_test!(dataview_resizable_with_data r#"const a = new DataView(new ArrayBuffer(2, { maxByteLength: 10 })); a.setUint8(0, 1); a"#);

// typed arrays
serde_test!(typed_array_inline r#"const a = new Float64Array(6); [a, new Uint8Array(a.buffer).subarray(3, 4)]"#);
serde_test!(typed_array_second_view_subarray r#"const a = new Uint8Array([1, 2]); [a, a.subarray(1, 2)]"#);
serde_test!(data_view_second_view r#"const a = new DataView(new Uint8Array([1, 2]).buffer); [a, new DataView(a.buffer, 1, 1)]"#);

// array buffer
serde_test!(array_buffer_empty r#"new ArrayBuffer()"#);
serde_test!(array_buffer_zeroed r#"new ArrayBuffer(5)"#);
serde_test!(array_buffer_data r#"const a = new Float64Array([1, 2, 3]); a.buffer"#);
serde_test!(indirect_array_buffer r#"const buf = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).buffer;
const foo = { buf };
[foo, new Uint8Array(buf)];
"#);
serde_test!(array_buffer_with_multiple_views r#"const buf = new Uint16Array([1, 2, 3]); [new Uint16Array(buf.buffer, 2, 1), new Int16Array(buf.buffer, 4, 1)]"#);

// circular reference
serde_test!(circular_reference r#"const foo = {}; foo.foo = foo; foo"#);
serde_test!(circular_reference_multi r#"const a = { b: {} }; a.b.a = a; a"#);
serde_test!(circular_more_complicated r#"const c = {}; const b = { a: {}, c }; b.a.c = c; b.a.b = b; b.a"#);

// error
serde_test!(error r#"new Error("foo", { cause: 1 })"#);
serde_test!(error_no_cause r#"new Error("foo")"#);
serde_test!(error_no_message r#"new Error(undefined)"#);
serde_test!(error_no_message_with_cause r#"new Error(undefined, { cause: 1 })"#);
serde_test!(error_no_stack r#"const err = new Error(); delete err.stack; err"#);
// V8 bug: https://bugs.chromium.org/p/v8/issues/detail?id=14433
// serde_test!(error_cause_self r#"const err = new Error("foo"); err.cause = err; err"#);
serde_test!(error_cause_multi_circular r#"const err = new Error("foo");  const a = { err }; err.cause = a; a"#);
serde_test!(error_prototype_eval r#"new EvalError("foo")"#);
serde_test!(error_prototype_range r#"new RangeError("foo")"#);
serde_test!(error_prototype_reference r#"new ReferenceError("foo")"#);
serde_test!(error_prototype_syntax r#"new SyntaxError("foo")"#);
serde_test!(error_prototype_type r#"new TypeError("foo")"#);
serde_test!(error_prototype_uri r#"new URIError("foo")"#);
serde_test!(error_cause_obj r#"new Error("foo", { cause: { a: 1 } })"#);
