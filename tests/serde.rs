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
// TODO: test utf8 string

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
serde_test!(dense_array_with_object r#"[1, { a: true }]"#);

// map
serde_test!(map_empty r#"new Map()"#);
serde_test!(map_one_element r#"new Map([["a", 1]])"#);
serde_test!(map_two_elements r#"new Map([["a", 1], ["b", 2]])"#);
serde_test!(map_object_key r#"new Map([[{ a: true }, 1]])"#);
serde_test!(map_object_value r#"new Map([[1, { b: true }]])"#);
serde_test!(map_object_key_and_value r#"new Map([[{ a: true }, { b: true }]])"#);

// set
serde_test!(set_empty r#"new Set()"#);
serde_test!(set_one_element r#"new Set([1])"#);
serde_test!(set_two_elements r#"new Set([1, 2])"#);
serde_test!(set_object_element r#"new Set([{ a: true }])"#);

// arraybuffer
serde_test!(arraybuffer_empty r#"new ArrayBuffer(0)"#);
serde_test!(arraybuffer_one_byte r#"new ArrayBuffer(1)"#);
serde_test!(arraybuffer_with_data r#"new Uint8Array([1,2]).buffer"#);

// resizable arraybuffer
serde_test!(resizable_arraybuffer_empty r#"new ArrayBuffer(2, { maxByteLength: 10 })"#);

// uint8array
serde_test!(uint8array_empty r#"new Uint8Array()"#);
serde_test!(uint8array_one_byte r#"new Uint8Array([1])"#);
serde_test!(uint8array_two_bytes r#"new Uint8Array([1, 2])"#);
serde_test!(uint8array_subarray r#"new Uint8Array([1, 2]).subarray(1, 2)"#);
serde_test!(uint8array_resizable r#"new Uint8Array(new ArrayBuffer(2, { maxByteLength: 10 }))"#);
serde_test!(uint8array_resizable_non_tracking r#"new Uint8Array(new ArrayBuffer(2, { maxByteLength: 10 })).subarray(0, 1)"#);

// uint8clampedarray
serde_test!(uint8clampedarray_empty r#"new Uint8ClampedArray()"#);
serde_test!(uint8clampedarray_one_byte r#"new Uint8ClampedArray([1])"#);
serde_test!(uint8clampedarray_two_bytes r#"new Uint8ClampedArray([1, 2])"#);
serde_test!(uint8clampedarray_subarray r#"new Uint8ClampedArray([1, 2]).subarray(1, 2)"#);
serde_test!(uint8clampedarray_resizable r#"new Uint8ClampedArray(new ArrayBuffer(2, { maxByteLength: 10 }))"#);
serde_test!(uint8clampedarray_resizable_non_tracking r#"new Uint8ClampedArray(new ArrayBuffer(2, { maxByteLength: 10 })).subarray(0, 1)"#);

// int8array
serde_test!(int8array_empty r#"new Int8Array()"#);
serde_test!(int8array_one_byte r#"new Int8Array([-1])"#);
serde_test!(int8array_two_bytes r#"new Int8Array([1, -2])"#);
serde_test!(int8array_subarray r#"new Int8Array([1, 2]).subarray(1, 2)"#);
serde_test!(int8array_resizable r#"new Int8Array(new ArrayBuffer(2, { maxByteLength: 10 }))"#);
serde_test!(int8array_resizable_non_tracking r#"new Int8Array(new ArrayBuffer(2, { maxByteLength: 10 })).subarray(0, 1)"#);

// uint16array
serde_test!(uint16array_empty r#"new Uint16Array()"#);
serde_test!(uint16array_one_byte r#"new Uint16Array([1])"#);
serde_test!(uint16array_two_bytes r#"new Uint16Array([1, 2])"#);
serde_test!(uint16array_subarray r#"new Uint16Array([1, 2]).subarray(1, 2)"#);
serde_test!(uint16array_resizable r#"new Uint16Array(new ArrayBuffer(2, { maxByteLength: 10 }))"#);
serde_test!(uint16array_resizable_non_tracking r#"new Uint16Array(new ArrayBuffer(4, { maxByteLength: 10 })).subarray(0, 1)"#);

// int16array
serde_test!(int16array_empty r#"new Int16Array()"#);
serde_test!(int16array_one_byte r#"new Int16Array([-1])"#);
serde_test!(int16array_two_bytes r#"new Int16Array([1, -2])"#);
serde_test!(int16array_subarray r#"new Int16Array([1, 2]).subarray(1, 2)"#);
serde_test!(int16array_resizable r#"new Int16Array(new ArrayBuffer(2, { maxByteLength: 10 }))"#);
serde_test!(int16array_resizable_non_tracking r#"new Uint16Array(new ArrayBuffer(4, { maxByteLength: 10 })).subarray(0, 1)"#);

// uint32array
serde_test!(uint32array_empty r#"new Uint32Array()"#);
serde_test!(uint32array_one_byte r#"new Uint32Array([1])"#);
serde_test!(uint32array_two_bytes r#"new Uint32Array([1, 2])"#);
serde_test!(uint32array_subarray r#"new Uint32Array([1, 2]).subarray(1, 2)"#);
serde_test!(uint32array_resizable r#"new Uint32Array(new ArrayBuffer(4, { maxByteLength: 12 }))"#);
serde_test!(uint32array_resizable_non_tracking r#"new Uint32Array(new ArrayBuffer(8, { maxByteLength: 12 })).subarray(0, 1)"#);

// int32array
serde_test!(int32array_empty r#"new Int32Array()"#);
serde_test!(int32array_one_byte r#"new Int32Array([-1])"#);
serde_test!(int32array_two_bytes r#"new Int32Array([1, -2])"#);
serde_test!(int32array_subarray r#"new Int32Array([1, 2]).subarray(1, 2)"#);
serde_test!(int32array_resizable r#"new Int32Array(new ArrayBuffer(4, { maxByteLength: 12 }))"#);
serde_test!(int32array_resizable_non_tracking r#"new Int32Array(new ArrayBuffer(8, { maxByteLength: 12 })).subarray(0, 1)"#);

// biguint64array
serde_test!(biguint64array_empty r#"new BigUint64Array()"#);
serde_test!(biguint64array_one_byte r#"new BigUint64Array([1n])"#);
serde_test!(biguint64array_two_bytes r#"new BigUint64Array([1n, 2n])"#);
serde_test!(biguint64array_subarray r#"new BigUint64Array([1n, 2n]).subarray(1, 2)"#);
serde_test!(biguint64array_resizable r#"new BigUint64Array(new ArrayBuffer(8, { maxByteLength: 16 }))"#);
serde_test!(biguint64array_resizable_non_tracking r#"new BigUint64Array(new ArrayBuffer(16, { maxByteLength: 24 })).subarray(0, 1)"#);

// bigint64array
serde_test!(bigint64array_empty r#"new BigInt64Array()"#);
serde_test!(bigint64array_one_byte r#"new BigInt64Array([-1n])"#);
serde_test!(bigint64array_two_bytes r#"new BigInt64Array([1n, -2n])"#);
serde_test!(bigint64array_subarray r#"new BigInt64Array([1n, 2n]).subarray(1, 2)"#);
serde_test!(bigint64array_resizable r#"new BigInt64Array(new ArrayBuffer(8, { maxByteLength: 16 }))"#);
serde_test!(bigint64array_resizable_non_tracking r#"new BigInt64Array(new ArrayBuffer(16, { maxByteLength: 24 })).subarray(0, 1)"#);

// circular reference
serde_test!(circular_reference r#"const foo = {}; foo.foo = foo; foo"#);
serde_test!(circular_reference_multi r#"const a = { b: {} }; a.b.a = a; a"#);

// error
serde_test!(error r#"new Error("foo", { cause: 1 })"#);
