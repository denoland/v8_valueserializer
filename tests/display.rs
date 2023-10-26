mod util;

display_test!(uint8array r#"new Uint8Array()"#);
display_test!(uint8array_zeroed r#"new Uint8Array(10)"#);
display_test!(uint8array_zeroed2 r#"new Uint8Array([0, 0, 0])"#);
display_test!(uint8array_data r#"new Uint8Array([0, 1, 0])"#);
display_test!(uint8array_data2 r#"new Uint8Array([0, 1, 2, 3, 4, 5, 6, 7, 8])"#);
display_test!(int8array_data r#"new Int8Array([0, -1, 2, -3])"#);
display_test!(biguint64array_data r#"new BigUint64Array([100000000000n, 1n, 2n, 3n, 4n, 5n, 6n, 7n, 8n])"#);
display_test!(float32array_zeroed r#"new Float64Array(6)"#);
display_test!(float32array_data r#"new Float64Array([NaN, 1, 2, 3, 4, 5, 6, 7, 8])"#);
display_test!(float64array_data r#"new Float64Array([0, 1, 2, 3, 4, 5, 6, 7, 8])"#);
display_test!(float64array_non_inline r#"const a = new Float64Array(6); [a, new Uint8Array(a.buffer).subarray(3, 4)]"#);
display_test!(arraybuffer r#"const a = new Float64Array([1, 2, 3]); a.buffer"#);
display_test!(resizable_arraybuffer r#"const a = new ArrayBuffer(5, { maxByteLength: 10 }); new Uint8Array(a).set([1, 2, 3, 4]); a"#);

display_test!(indirect_uint8array r#"const buf = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).buffer;
const foo = { buf };
[foo, new Uint8Array(buf)];
"#);

display_test!(circular_reference r#"const a = {}; const b = new Array(2); b.a = a; a.b = b; a"#);
display_test!(circular_reference_ordering r#"const a = {}; const b = new Array(2); b.a = a; b.x = 1; a.b = b; a"#);

display_test!(error r#"const err = new Error(undefined); err.cause = err; err"#);

display_test!(regexp r#"/foo/mg"#);

display_test!(onebyte r#""è÷\u{0}""#);
