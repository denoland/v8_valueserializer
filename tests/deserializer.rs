mod util;

// special values
deserialize_test!(undefined r#"undefined"#);
deserialize_test!(null r#"null"#);

// booleans
deserialize_test!(bool_true r#"true"#);
deserialize_test!(bool_false r#"false"#);

// small integers
deserialize_test!(i32 r#"1"#);
deserialize_test!(i32_neg r#"1000"#);
deserialize_test!(i32_zero r#"0"#);
deserialize_test!(i32_max r#"2147483647"#);
deserialize_test!(i32_min r#"-2147483648"#);

// doubles
deserialize_test!(double r#"1.1"#);
deserialize_test!(double_neg r#"-1.1"#);
deserialize_test!(double_neg_zero r#"-0"#);
deserialize_test!(double_nan r#"NaN"#);
deserialize_test!(double_infinity r#"Infinity"#);
deserialize_test!(double_neg_infinity r#"-Infinity"#);
deserialize_test!(double_max r#"1.7976931348623157e+308"#);
deserialize_test!(double_min r#"5e-324"#);

// bigints
deserialize_test!(bigint r#"1n"#);
deserialize_test!(bigint_neg r#"-1n"#);
deserialize_test!(bigint_zero r#"0n"#);
deserialize_test!(bigint_u64_max r#"18446744073709551615n"#);
deserialize_test!(bigint_u128_max r#"340282366920938463463374607431768211455n"#);

// strings
deserialize_test!(string_empty r#"''"#);
deserialize_test!(string_one_byte r#"'asd'"#);
deserialize_test!(string_two_byte r#"'asd 🌎'"#);
// TODO: test utf8 string

// boolean primitive wrapper object
deserialize_test!(boolean_primitive_true r#"new Boolean(true)"#);
deserialize_test!(boolean_primitive_false r#"new Boolean(false)"#);

// number primitive wrapper object
deserialize_test!(number_primitive r#"new Number(1)"#);
deserialize_test!(number_primitive_neg r#"new Number(-1)"#);
deserialize_test!(number_primitive_zero r#"new Number(0)"#);
deserialize_test!(number_primitive_nan r#"new Number(NaN)"#);
deserialize_test!(number_primitive_infinity r#"new Number(Infinity)"#);
deserialize_test!(number_primitive_infinity_neg r#"new Number(-Infinity)"#);
deserialize_test!(number_primitive_max r#"new Number(1.7976931348623157e+308)"#);
deserialize_test!(number_primitive_min r#"new Number(5e-324)"#);

// bigint primitive wrapper object
deserialize_test!(bigint_primitive r#"Object(1n)"#);
deserialize_test!(bigint_primitive_neg r#"Object(-1n)"#);
deserialize_test!(bigint_primitive_zero r#"Object(0n)"#);
deserialize_test!(bigint_primitive_u64_max r#"Object(18446744073709551615n)"#);
deserialize_test!(bigint_primitive_u128_max r#"Object(340282366920938463463374607431768211455n)"#);

// string primitive wrapper object
deserialize_test!(string_primitive_empty r#"new String('')"#);
deserialize_test!(string_primitive_one_byte r#"new String('asd')"#);
deserialize_test!(string_primitive_two_byte r#"new String('asd 🌎')"#);

// regexp
deserialize_test!(regexp r#"/asd/gi"#);
deserialize_test!(regexp_empty r#"new RegExp("(?:)")"#);
deserialize_test!(regexp_two_byte r#"/🗄️/"#);

// date
deserialize_test!(date r#"new Date(1)"#);
deserialize_test!(date_zero r#"new Date(0)"#);
deserialize_test!(date_max r#"new Date(8640000000000000)"#);
deserialize_test!(date_min r#"new Date(-8640000000000000)"#);
deserialize_test!(date_invalid r#"new Date(NaN)"#);

// object
deserialize_test!(object_empty r#"{}"#);
deserialize_test!(object_one_property r#"{"a": 1}"#);
deserialize_test!(object_two_properties r#"{"a": 1, "b": 2}"#);
deserialize_test!(object_smi_property_key r#"{[1]: 3}"#);
deserialize_test!(object_smi_property_key_neg r#"{[-1]: 4}"#);
deserialize_test!(object_smi_property_key_as_str r#"{"1": 5}"#);
deserialize_test!(object_two_byte_property_key r#"{"foo🗄️": 6}"#);
deserialize_test!(object_nested r#"{"a": {"b": true}}"#);

// sparse array
deserialize_test!(sparse_array_empty r#"new Array(0)"#);
deserialize_test!(sparse_array_empty_length_one r#"new Array(1)"#);
deserialize_test!(sparse_array_one_element_length_one r#"
const arr = new Array(1);
arr[0] = 1;
arr
"#);
deserialize_test!(sparse_array_one_element_length_two r#"
const arr = new Array(2);
arr[1] = 1;
arr
"#);
deserialize_test!(sparse_array_one_element_length_two_with_properties r#"
const arr = new Array();
arr[1] = 1;
arr.foo = "bar";
arr
"#);
deserialize_test!(sparse_array_literal_with_hole r#"[1, 2, /* hole */, 4]"#);
deserialize_test!(sparse_array_with_object r#"[/* hole */, {a: 2}]"#);

// dense array
deserialize_test!(dense_array r#"[1, 2]"#);
deserialize_test!(dense_array_empty r#"[]"#);
deserialize_test!(dense_array_one_element r#"[1]"#);
deserialize_test!(dense_array_two_elements_multi_type r#"[1, "asd"]"#);
deserialize_test!(dense_array_with_properties r#"
const arr = ["asd", 1];
arr.foo = "bar";
arr
"#);
deserialize_test!(dense_array_with_object r#"[1, { a: true }]"#);

// map
deserialize_test!(map_empty r#"new Map()"#);
deserialize_test!(map_one_element r#"new Map([["a", 1]])"#);
deserialize_test!(map_two_elements r#"new Map([["a", 1], ["b", 2]])"#);
deserialize_test!(map_object_key r#"new Map([[{ a: true }, 1]])"#);
deserialize_test!(map_object_value r#"new Map([[1, { b: true }]])"#);
deserialize_test!(map_object_key_and_value r#"new Map([[{ a: true }, { b: true }]])"#);

// set
deserialize_test!(set_empty r#"new Set()"#);
deserialize_test!(set_one_element r#"new Set([1])"#);
deserialize_test!(set_two_elements r#"new Set([1, 2])"#);
deserialize_test!(set_object_element r#"new Set([{ a: true }])"#);

// arraybuffer
deserialize_test!(arraybuffer_empty r#"new ArrayBuffer(0)"#);
deserialize_test!(arraybuffer_one_byte r#"new ArrayBuffer(1)"#);
deserialize_test!(arraybuffer_with_data r#"new Uint8Array([1,2]).buffer"#);

// resizable arraybuffer
deserialize_test!(resizable_arraybuffer_empty r#"new ArrayBuffer(2, { maxByteLength: 10 })"#);

// uint8array
deserialize_test!(uint8array_empty r#"new Uint8Array()"#);
deserialize_test!(uint8array_one_byte r#"new Uint8Array([1])"#);
deserialize_test!(uint8array_two_bytes r#"new Uint8Array([1, 2])"#);
deserialize_test!(uint8array_resizable r#"new Uint8Array(new ArrayBuffer(2, { maxByteLength: 10 }))"#);
deserialize_test!(uint8array_resizable_non_tracking r#"new Uint8Array(new ArrayBuffer(2, { maxByteLength: 10 })).subarray(0, 1)"#);

// uint8clampedarray
deserialize_test!(uint8clampedarray_empty r#"new Uint8ClampedArray()"#);
deserialize_test!(uint8clampedarray_one_byte r#"new Uint8ClampedArray([1])"#);
deserialize_test!(uint8clampedarray_two_bytes r#"new Uint8ClampedArray([1, 2])"#);
deserialize_test!(uint8clampedarray_resizable r#"new Uint8ClampedArray(new ArrayBuffer(2, { maxByteLength: 10 }))"#);
deserialize_test!(uint8clampedarray_resizable_non_tracking r#"new Uint8ClampedArray(new ArrayBuffer(2, { maxByteLength: 10 })).subarray(0, 1)"#);

// int8array
deserialize_test!(int8array_empty r#"new Int8Array()"#);
deserialize_test!(int8array_one_byte r#"new Int8Array([-1])"#);
deserialize_test!(int8array_two_bytes r#"new Int8Array([1, -2])"#);
deserialize_test!(int8array_resizable r#"new Int8Array(new ArrayBuffer(2, { maxByteLength: 10 }))"#);
deserialize_test!(int8array_resizable_non_tracking r#"new Int8Array(new ArrayBuffer(2, { maxByteLength: 10 })).subarray(0, 1)"#);

// uint16array
deserialize_test!(uint16array_empty r#"new Uint16Array()"#);
deserialize_test!(uint16array_one_byte r#"new Uint16Array([1])"#);
deserialize_test!(uint16array_two_bytes r#"new Uint16Array([1, 2])"#);
deserialize_test!(uint16array_resizable r#"new Uint16Array(new ArrayBuffer(2, { maxByteLength: 10 }))"#);
deserialize_test!(uint16array_resizable_non_tracking r#"new Uint16Array(new ArrayBuffer(4, { maxByteLength: 10 })).subarray(0, 1)"#);

// int16array
deserialize_test!(int16array_empty r#"new Int16Array()"#);
deserialize_test!(int16array_one_byte r#"new Int16Array([-1])"#);
deserialize_test!(int16array_two_bytes r#"new Int16Array([1, -2])"#);
deserialize_test!(int16array_resizable r#"new Int16Array(new ArrayBuffer(2, { maxByteLength: 10 }))"#);
deserialize_test!(int16array_resizable_non_tracking r#"new Uint16Array(new ArrayBuffer(4, { maxByteLength: 10 })).subarray(0, 1)"#);

// uint32array
deserialize_test!(uint32array_empty r#"new Uint32Array()"#);
deserialize_test!(uint32array_one_byte r#"new Uint32Array([1])"#);
deserialize_test!(uint32array_two_bytes r#"new Uint32Array([1, 2])"#);
deserialize_test!(uint32array_resizable r#"new Uint32Array(new ArrayBuffer(4, { maxByteLength: 12 }))"#);
deserialize_test!(uint32array_resizable_non_tracking r#"new Uint32Array(new ArrayBuffer(8, { maxByteLength: 12 })).subarray(0, 1)"#);

// int32array
deserialize_test!(int32array_empty r#"new Int32Array()"#);
deserialize_test!(int32array_one_byte r#"new Int32Array([-1])"#);
deserialize_test!(int32array_two_bytes r#"new Int32Array([1, -2])"#);
deserialize_test!(int32array_resizable r#"new Int32Array(new ArrayBuffer(4, { maxByteLength: 12 }))"#);
deserialize_test!(int32array_resizable_non_tracking r#"new Int32Array(new ArrayBuffer(8, { maxByteLength: 12 })).subarray(0, 1)"#);

// biguint64array
deserialize_test!(biguint64array_empty r#"new BigUint64Array()"#);
deserialize_test!(biguint64array_one_byte r#"new BigUint64Array([1n])"#);
deserialize_test!(biguint64array_two_bytes r#"new BigUint64Array([1n, 2n])"#);
deserialize_test!(biguint64array_resizable r#"new BigUint64Array(new ArrayBuffer(8, { maxByteLength: 16 }))"#);
deserialize_test!(biguint64array_resizable_non_tracking r#"new BigUint64Array(new ArrayBuffer(16, { maxByteLength: 24 })).subarray(0, 1)"#);

// bigint64array
deserialize_test!(bigint64array_empty r#"new BigInt64Array()"#);
deserialize_test!(bigint64array_one_byte r#"new BigInt64Array([-1n])"#);
deserialize_test!(bigint64array_two_bytes r#"new BigInt64Array([1n, -2n])"#);
deserialize_test!(bigint64array_resizable r#"new BigInt64Array(new ArrayBuffer(8, { maxByteLength: 16 }))"#);
deserialize_test!(bigint64array_resizable_non_tracking r#"new BigInt64Array(new ArrayBuffer(16, { maxByteLength: 24 })).subarray(0, 1)"#);

// circular reference
deserialize_test!(circular_reference r#"const foo = {}; foo.foo = foo; foo"#);
deserialize_test!(circular_reference_multi r#"const a = { b: {} }; a.b.a = a; a"#);
