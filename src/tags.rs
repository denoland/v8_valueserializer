/// https://source.chromium.org/chromium/chromium/src/+/main:v8/src/objects/value-serializer.cc;drc=f5bdc89c7395ed24f1b8d196a3bdd6232d5bf771;bpv=0;bpt=1;l=93
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SerializationTag {
  // version:u32 (if at beginning of data, sets version > 0)
  Version = 0xFF,
  // ignore
  Padding = 0x00,
  // refTableSize:u32 (previously used for sanity checks; safe to ignore)
  VerifyObjectCount = b'?',
  // Oddballs (no data).
  TheHole = b'-',
  Undefined = b'_',
  Null = b'0',
  True = b'T',
  False = b'F',
  // Number represented as 32-bit integer, ZigZag-encoded
  // (like sint32 in protobuf)
  Int32 = b'I',
  // Number represented as 32-bit unsigned integer, varint-encoded
  // (like uint32 in protobuf)
  Uint32 = b'U',
  // Number represented as a 64-bit double.
  // Host byte order is used (N.B. this makes the format non-portable).
  Double = b'N',
  // BigInt. Bitfield:u32, then raw digits storage.
  BigInt = b'Z',
  // byteLength:u32, then raw data
  Utf8String = b'S',
  OneByteString = b'"',
  TwoByteString = b'c',
  // Reference to a serialized object. objectID:u32
  ObjectReference = b'^',
  // Beginning of a JS object.
  BeginJsObject = b'o',
  // End of a JS object. numProperties:u32
  EndJsObject = b'{',
  // Beginning of a sparse JS array. length:u32
  // Elements and properties are written as key/value pairs, like objects.
  BeginSparseJsArray = b'a',
  // End of a sparse JS array. numProperties:u32 length:u32
  EndSparseJsArray = b'@',
  // Beginning of a dense JS array. length:u32
  // |length| elements, followed by properties as key/value pairs
  BeginDenseJsArray = b'A',
  // End of a dense JS array. numProperties:u32 length:u32
  EndDenseJsArray = b'$',
  // Date. millisSinceEpoch:double
  Date = b'D',
  // Boolean object. No data.
  TrueObject = b'y',
  FalseObject = b'x',
  // Number object. value:double
  NumberObject = b'n',
  // BigInt object. Bitfield:u32, then raw digits storage.
  BigIntObject = b'z',
  // String object, UTF-8 encoding. byteLength:u32, then raw data.
  StringObject = b's',
  // Regular expression, UTF-8 encoding. byteLength:u32, raw data,
  // flags:u32.
  RegExp = b'R',
  // Beginning of a JS map.
  BeginJsMap = b';',
  // End of a JS map. length:u32.
  EndJsMap = b':',
  // Beginning of a JS set.
  BeginJsSet = b'\'',
  // End of a JS set. length:u32.
  EndJsSet = b',',
  // Array buffer. byteLength:u32, then raw data.
  ArrayBuffer = b'B',
  // Resizable ArrayBuffer.
  ResizableArrayBuffer = b'~',
  // Array buffer (transferred). transferID:u32
  ArrayBufferTransfer = b't',
  // View into an array buffer.
  // subtag:ArrayBufferViewTag, byteOffset:u32, byteLength:u32
  // For typed arrays, byteOffset and byteLength must be divisible by the size
  // of the element.
  // Note: kArrayBufferView is special, and should have an ArrayBuffer (or an
  // ObjectReference to one) serialized just before it. This is a quirk arising
  // from the previous stack-based implementation.
  ArrayBufferView = b'V',
  // Shared array buffer. transferID:u32
  SharedArrayBuffer = b'u',
  // A HeapObject shared across Isolates. sharedValueID:u32
  SharedObject = b'p',
  // A wasm module object transfer. next value is its index.
  WasmModuleTransfer = b'w',
  // The delegate is responsible for processing all following data.
  // This "escapes" to whatever wire format the delegate chooses.
  HostObject = b'\\',
  // A transferred WebAssembly.Memory object. maximumPages:i32, then by
  // SharedArrayBuffer tag and its data.
  WasmMemoryTransfer = b'm',
  // A list of (subtag: ErrorTag, [subtag dependent data]). See ErrorTag for
  // details.
  Error = b'r',
}

/// https://source.chromium.org/chromium/chromium/src/+/main:v8/src/objects/value-serializer.cc;l=93;drc=f5bdc89c7395ed24f1b8d196a3bdd6232d5bf771;bpv=1;bpt=1

#[repr(u8)]
pub enum ArrayBufferViewTag {
  Int8Array = b'b',
  Uint8Array = b'B',
  Uint8ClampedArray = b'C',
  Int16Array = b'w',
  Uint16Array = b'W',
  Int32Array = b'd',
  Uint32Array = b'D',
  Float32Array = b'f',
  Float64Array = b'F',
  BigInt64Array = b'q',
  BigUint64Array = b'Q',
  DataView = b'?',
}

// /// https://source.chromium.org/chromium/chromium/src/+/main:v8/src/objects/value-serializer.cc;l=93;drc=f5bdc89c7395ed24f1b8d196a3bdd6232d5bf771;bpv=1;bpt=1
// pub struct ErrorTag(u8);

// impl ErrorTag {
//   pub const EVAL_ERROR_PROTOTYPE: Self = Self(b'E');
//   // The error is a RangeError. No accompanying data.
//   pub const RANGE_ERROR_PROTOTYPE: Self = Self(b'R');
//   // The error is a ReferenceError. No accompanying data.
//   pub const REFERENCE_ERROR_PROTOTYPE: Self = Self(b'F');
//   // The error is a SyntaxError. No accompanying data.
//   pub const SYNTAX_ERROR_PROTOTYPE: Self = Self(b'S');
//   // The error is a TypeError. No accompanying data.
//   pub const TYPE_ERROR_PROTOTYPE: Self = Self(b'T');
//   // The error is a URIError. No accompanying data.
//   pub const URI_ERROR_PROTOTYPE: Self = Self(b'U');
//   // Followed by message: string.
//   pub const MESSAGE: Self = Self(b'm');
//   // Followed by a JS object: cause.
//   pub const CAUSE: Self = Self(b'c');
//   // Followed by stack: string.
//   pub const STACK: Self = Self(b's');
//   // The end of this error information.
//   pub const END: Self = Self(b'.');
// }
