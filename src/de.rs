use num_bigint::BigInt;
use std::alloc::Layout;
use std::collections::HashMap;
use std::mem::align_of;
use std::mem::size_of;
use thiserror::Error;

use crate::tags::ArrayBufferViewTag;
use crate::tags::ErrorTag;
use crate::tags::SerializationTag;
use crate::value::ArrayBuffer;
use crate::value::ArrayBufferView;
use crate::value::ArrayBufferViewKind;
use crate::value::Date;
use crate::value::DenseArray;
use crate::value::Error;
use crate::value::ErrorName;
use crate::value::Heap;
use crate::value::HeapBuilder;
use crate::value::HeapValue;
use crate::value::Map;
use crate::value::Object;
use crate::value::OneByteString;
use crate::value::PropertyKey;
use crate::value::RegExp;
use crate::value::RegExpFlags;
use crate::value::Set;
use crate::value::SparseArray;
use crate::value::Value;
use crate::value::Wtf8String;
use crate::HeapReference;
use crate::StringValue;
use crate::TwoByteString;

const MINIMUM_WIRE_FORMAT_VERSION: u32 = 14;
const MAXIMUM_WIRE_FORMAT_VERSION: u32 = 15;

#[derive(Debug, Error)]
#[error("parse error at position {position}: {kind}")]
pub struct ParseError {
  position: usize,
  pub kind: ParseErrorKind,
}

#[derive(Debug, Error)]
pub enum ParseErrorKind {
  #[error("unexpected end of file")]
  UnexpectedEof,
  #[error("expected at least {0} more bytes, but only {1} bytes are left")]
  ExpectedMinimumBytes(usize, usize),
  #[error("expected tag {0:?} but got {1:?}")]
  ExpectedTag(SerializationTag, u8),
  #[error("invalid wire format version")]
  InvalidWireFormatVersion(u32),
  #[error("unexpected tag {0:?}")]
  UnexpectedTag(u8),
  #[error("unexpected error tag {0:?}")]
  UnexpectedErrorTag(u8),
  #[error("invalid one byte string")]
  InvalidLengthTwoByteString,
  #[error("invalid object reference {0}")]
  InvalidObjectReference(u32),
  #[error(
    "invalid array elements length: expected {expected}, actual {actual}"
  )]
  InvalidArrayElementsLength { expected: u32, actual: u32 },
  #[error("invalid property count: expected {expected}, actual {actual}")]
  InvalidPropertyCount { expected: u32, actual: u32 },
  #[error("invalid map/set entry count: expected {expected}, actual {actual}")]
  InvalidEntryCount { expected: u32, actual: u32 },
  #[error("invalid property key: {0:?}")]
  InvalidPropertyKey(Value),
  #[error("failed to build heap: {0}")]
  HeapBuildError(#[from] crate::value::HeapBuildError),
  #[error("resizable array buffer max length is shorter than length: actual: {byte_length}, max: {max_byte_length}")]
  InvalidResizableArrayBufferMaxLength {
    byte_length: u32,
    max_byte_length: u32,
  },
  #[error("expected end of file")]
  ExpectedEof,
  #[error("invalid array buffer view offset: byte offset: {byte_offset}, buffer byte length: {buffer_byte_length}")]
  InvalidArrayBufferViewOffset {
    byte_offset: u32,
    buffer_byte_length: u32,
  },
  #[error("invalid array buffer view length: byte length: {byte_length}, byte offset: {byte_offset}, buffer byte length: {buffer_byte_length}")]
  InvalidArrayBufferViewLength {
    byte_length: u32,
    byte_offset: u32,
    buffer_byte_length: u32,
  },
  #[error("invalid array buffer view tag: {0}")]
  InvalidArrayBufferViewTag(u8),
  #[error("unaligned array buffer view offset: byte offset: {byte_offset}, element size: {element_size}")]
  UnalignedArrayBufferViewOffset { byte_offset: u32, element_size: u32 },
  #[error("unaligned array buffer view length: byte length: {byte_length}, element size: {element_size}")]
  UnalignedArrayBufferViewLength { byte_length: u32, element_size: u32 },
  #[error("shared array buffers (and by extension Wasm memory objects) are not supported")]
  SharedArrayBufferNotSupported,
  #[error("missing transferred array buffer {}", .0)]
  MissingTransferredArrayBuffer(u32),
  #[error("wasm module object transfers are not supported")]
  WasmModuleTransferNotSupported,
  #[error("host objects are not supported")]
  HostObjectNotSupported,
  #[error("shared objects are not supported")]
  SharedObjectNotSupported,
  #[error("an object is too deeply nested, hit recursion depth limit")]
  TooDeeplyNested,
  #[error("invalid regexp flags: {:b}", .0)]
  InvalidRegExpFlags(u32),
}

struct Input<'a> {
  bytes: &'a [u8],
  position: usize,
}

#[derive(Default)]
pub struct ValueDeserializer {
  transfer_map: HashMap<u32, ArrayBuffer>,
  recursion_depth: usize,
}

impl ValueDeserializer {
  pub fn transfer_array_buffer(&mut self, id: u32, ab: ArrayBuffer) {
    self.transfer_map.insert(id, ab);
  }

  pub fn read(mut self, bytes: &[u8]) -> Result<(Value, Heap), ParseError> {
    let mut input = Input { bytes, position: 0 };
    input.expect_tag(SerializationTag::Version)?;
    let version = input.read_varint()?;
    if !(MINIMUM_WIRE_FORMAT_VERSION..=MAXIMUM_WIRE_FORMAT_VERSION)
      .contains(&version)
    {
      return Err(input.err(ParseErrorKind::InvalidWireFormatVersion(version)));
    }
    let mut heap_builder = HeapBuilder::default();
    let value = read_object(&mut self, &mut input, &mut heap_builder)?;
    input.expect_eof()?;
    let heap = heap_builder
      .build()
      .map_err(|err| input.err_current(err.into()))?;
    Ok((value, heap))
  }
}

const RECURSION_DEPTH_LIMIT: usize = 256;

fn read_object(
  de: &mut ValueDeserializer,
  input: &mut Input<'_>,
  heap: &mut HeapBuilder,
) -> Result<Value, ParseError> {
  if de.recursion_depth > RECURSION_DEPTH_LIMIT {
    return Err(input.err(ParseErrorKind::TooDeeplyNested));
  }
  de.recursion_depth += 1;
  let res = read_object_internal(de, input, heap);
  de.recursion_depth -= 1;
  let value = res?;

  if let Value::HeapReference(reference) = value {
    match heap.try_open(reference) {
      Some(HeapValue::ArrayBuffer(ab))
        if input.maybe_read_tag(SerializationTag::ArrayBufferView) =>
      {
        let buffer_byte_length = ab.byte_length();
        let view =
          read_js_array_buffer_view(input, buffer_byte_length, reference)?;
        let heap_value = HeapValue::ArrayBufferView(view);
        let reference = heap.insert(heap_value);
        return Ok(Value::HeapReference(reference));
      }
      _ => {}
    };
  }

  Ok(value)
}

fn read_object_internal(
  de: &mut ValueDeserializer,
  input: &mut Input<'_>,
  heap: &mut HeapBuilder,
) -> Result<Value, ParseError> {
  input.skip_padding();
  let tag = input.read_byte()?;
  if tag == SerializationTag::VerifyObjectCount as u8 {
    // Read the count and ignore it.
    let _ = input.read_varint()?;
    read_object(de, input, heap)
  } else if tag == SerializationTag::Undefined as u8 {
    Ok(Value::Undefined)
  } else if tag == SerializationTag::Null as u8 {
    Ok(Value::Null)
  } else if tag == SerializationTag::True as u8 {
    Ok(Value::Bool(true))
  } else if tag == SerializationTag::False as u8 {
    Ok(Value::Bool(false))
  } else if tag == SerializationTag::Int32 as u8 {
    let value = input.read_zigzag()?;
    Ok(Value::I32(value))
  } else if tag == SerializationTag::Uint32 as u8 {
    let value = input.read_varint()?;
    Ok(Value::U32(value))
  } else if tag == SerializationTag::Double as u8 {
    let value = input.read_double()?;
    Ok(Value::Double(value))
  } else if tag == SerializationTag::BigInt as u8 {
    let value = read_bigint(input)?;
    Ok(Value::BigInt(value))
  } else if tag == SerializationTag::Utf8String as u8 {
    let value = read_utf8_string(input)?;
    Ok(Value::String(StringValue::Wtf8(value)))
  } else if tag == SerializationTag::OneByteString as u8 {
    let value = read_one_byte_string(input)?;
    Ok(Value::String(StringValue::OneByte(value)))
  } else if tag == SerializationTag::TwoByteString as u8 {
    let str = read_two_byte_string(input)?;
    Ok(Value::String(StringValue::TwoByte(str)))
  } else if tag == SerializationTag::ObjectReference as u8 {
    let reference = read_object_reference(input, heap)?;
    Ok(Value::HeapReference(reference))
  } else if tag == SerializationTag::BeginJsObject as u8 {
    let reference = heap.reserve();
    let object = read_js_object(de, input, heap)?;
    let heap_value = HeapValue::Object(object);
    heap.insert_reserved(reference, heap_value);
    Ok(Value::HeapReference(reference))
  } else if tag == SerializationTag::BeginSparseJsArray as u8 {
    let reference = heap.reserve();
    let array = read_sparse_js_array(de, input, heap)?;
    let heap_value = HeapValue::SparseArray(array);
    heap.insert_reserved(reference, heap_value);
    Ok(Value::HeapReference(reference))
  } else if tag == SerializationTag::BeginDenseJsArray as u8 {
    let reference = heap.reserve();
    let array = read_dense_js_array(de, input, heap)?;
    let heap_value = HeapValue::DenseArray(array);
    heap.insert_reserved(reference, heap_value);
    Ok(Value::HeapReference(reference))
  } else if tag == SerializationTag::Date as u8 {
    let date = read_date(input)?;
    let heap_value = HeapValue::Date(date);
    let reference = heap.insert(heap_value);
    Ok(Value::HeapReference(reference))
  } else if tag == SerializationTag::TrueObject as u8 {
    let heap_value = HeapValue::BooleanObject(true);
    let reference = heap.insert(heap_value);
    Ok(Value::HeapReference(reference))
  } else if tag == SerializationTag::FalseObject as u8 {
    let heap_value = HeapValue::BooleanObject(false);
    let reference = heap.insert(heap_value);
    Ok(Value::HeapReference(reference))
  } else if tag == SerializationTag::NumberObject as u8 {
    let value = input.read_double()?;
    let heap_value = HeapValue::NumberObject(value);
    let reference = heap.insert(heap_value);
    Ok(Value::HeapReference(reference))
  } else if tag == SerializationTag::BigIntObject as u8 {
    let value = read_bigint(input)?;
    let heap_value = HeapValue::BigIntObject(value);
    let reference = heap.insert(heap_value);
    Ok(Value::HeapReference(reference))
  } else if tag == SerializationTag::StringObject as u8 {
    let value = read_string_value(de, input)?;
    let heap_value = HeapValue::StringObject(value);
    let reference = heap.insert(heap_value);
    Ok(Value::HeapReference(reference))
  } else if tag == SerializationTag::RegExp as u8 {
    let regexp = read_regexp(de, input)?;
    let heap_value = HeapValue::RegExp(regexp);
    let reference = heap.insert(heap_value);
    Ok(Value::HeapReference(reference))
  } else if tag == SerializationTag::BeginJsMap as u8 {
    let reference = heap.reserve();
    let map = read_js_map(de, input, heap)?;
    let heap_value = HeapValue::Map(map);
    heap.insert_reserved(reference, heap_value);
    Ok(Value::HeapReference(reference))
  } else if tag == SerializationTag::BeginJsSet as u8 {
    let reference = heap.reserve();
    let set = read_js_set(de, input, heap)?;
    let heap_value = HeapValue::Set(set);
    heap.insert_reserved(reference, heap_value);
    Ok(Value::HeapReference(reference))
  } else if tag == SerializationTag::ArrayBuffer as u8 {
    let array_buffer = read_js_array_buffer(input, false)?;
    let heap_value = HeapValue::ArrayBuffer(array_buffer);
    let reference = heap.insert(heap_value);
    Ok(Value::HeapReference(reference))
  } else if tag == SerializationTag::ResizableArrayBuffer as u8 {
    let array_buffer = read_js_array_buffer(input, true)?;
    let heap_value = HeapValue::ArrayBuffer(array_buffer);
    let reference = heap.insert(heap_value);
    Ok(Value::HeapReference(reference))
  } else if tag == SerializationTag::ArrayBufferTransfer as u8 {
    let array_buffer = read_transferred_js_array_buffer(de, input)?;
    let heap_value = HeapValue::ArrayBuffer(array_buffer);
    let reference = heap.insert(heap_value);
    Ok(Value::HeapReference(reference))
  } else if tag == SerializationTag::SharedArrayBuffer as u8 {
    Err(input.err(ParseErrorKind::SharedArrayBufferNotSupported))
  } else if tag == SerializationTag::Error as u8 {
    let reference = heap.reserve();
    let error = read_js_error(de, input, heap)?;
    let heap_value = HeapValue::Error(error);
    heap.insert_reserved(reference, heap_value);
    Ok(Value::HeapReference(reference))
  } else if tag == SerializationTag::WasmModuleTransfer as u8 {
    Err(input.err(ParseErrorKind::WasmModuleTransferNotSupported))
  } else if tag == SerializationTag::WasmMemoryTransfer as u8 {
    Err(input.err(ParseErrorKind::SharedArrayBufferNotSupported))
  } else if tag == SerializationTag::HostObject as u8 {
    Err(input.err(ParseErrorKind::HostObjectNotSupported))
  } else if tag == SerializationTag::SharedObject as u8 {
    Err(input.err(ParseErrorKind::SharedObjectNotSupported))
  } else {
    Err(input.err(ParseErrorKind::UnexpectedTag(tag)))
  }
}

fn read_bigint(input: &mut Input<'_>) -> Result<BigInt, ParseError> {
  const BIGINT_SIGN_BIT_MASK: u32 = 1;
  const BIGINT_BYTE_LENGTH_MASK: u32 = 0x7FFFFFFE;
  // This bitfield stores both the sign (least significant bit) and the byte
  // length (next 30 bits). The final (most significant) bit is currently
  // unused.
  let bitfield = input.read_varint()?;
  let sign = if bitfield & BIGINT_SIGN_BIT_MASK == 0 {
    num_bigint::Sign::Plus
  } else {
    num_bigint::Sign::Minus
  };
  let byte_length = ((bitfield & BIGINT_BYTE_LENGTH_MASK) >> 1) as usize;
  let bytes = input.read_bytes(byte_length)?;
  Ok(BigInt::from_bytes_le(sign, bytes))
}

fn read_utf8_string(input: &mut Input<'_>) -> Result<Wtf8String, ParseError> {
  let byte_length = input.read_varint()?;
  let bytes = input.read_bytes(byte_length as usize)?;
  let string = Wtf8String::new(bytes.to_vec());
  Ok(string)
}

fn read_one_byte_string(
  input: &mut Input<'_>,
) -> Result<OneByteString, ParseError> {
  let byte_length = input.read_varint()?;
  let bytes = input.read_bytes(byte_length as usize)?;
  let string = OneByteString::new(bytes.to_vec());
  Ok(string)
}

fn read_two_byte_string(
  input: &mut Input<'_>,
) -> Result<TwoByteString, ParseError> {
  let byte_length = input.read_varint()?;
  if byte_length % 2 != 0 {
    return Err(input.err(ParseErrorKind::InvalidLengthTwoByteString));
  }
  let bytes = input.read_bytes(byte_length as usize)?;
  // This allocation is not unbounded, because it will only occur if the input
  // contained at least byte_length bytes.
  let mut chars = vec![0u16; byte_length as usize / 2];
  // Safety: we checked that bytes is a multiple of 2, and chars has the same
  // length as bytes divided by 2. Therefore, the length of bytes and chars is
  // the same, and we can copy bytes into chars.
  unsafe {
    std::ptr::copy_nonoverlapping(
      bytes.as_ptr(),
      chars.as_mut_ptr() as *mut u8,
      byte_length as usize,
    )
  };
  Ok(TwoByteString::new(chars))
}

fn read_string_value(
  de: &mut ValueDeserializer,
  input: &mut Input<'_>,
) -> Result<StringValue, ParseError> {
  if de.recursion_depth > RECURSION_DEPTH_LIMIT {
    return Err(input.err(ParseErrorKind::TooDeeplyNested));
  }
  input.skip_padding();
  let tag = input.read_byte()?;
  if tag == SerializationTag::VerifyObjectCount as u8 {
    // Read the count and ignore it.
    let _ = input.read_varint()?;
    de.recursion_depth += 1;
    let res = read_string_value(de, input);
    de.recursion_depth -= 1;
    res
  } else if tag == SerializationTag::Utf8String as u8 {
    let value = read_utf8_string(input)?;
    Ok(StringValue::Wtf8(value))
  } else if tag == SerializationTag::OneByteString as u8 {
    let value = read_one_byte_string(input)?;
    Ok(StringValue::OneByte(value))
  } else if tag == SerializationTag::TwoByteString as u8 {
    let value = read_two_byte_string(input)?;
    Ok(StringValue::TwoByte(value))
  } else {
    Err(input.err(ParseErrorKind::UnexpectedTag(tag)))
  }
}

fn read_object_reference(
  input: &mut Input<'_>,
  heap: &mut HeapBuilder,
) -> Result<HeapReference, ParseError> {
  let index = input.read_varint()?;
  heap
    .reference_by_id(index)
    .ok_or_else(|| input.err(ParseErrorKind::InvalidObjectReference(index)))
}

fn read_js_object_properties(
  de: &mut ValueDeserializer,
  input: &mut Input<'_>,
  heap: &mut HeapBuilder,
  end_tag: SerializationTag,
) -> Result<Vec<(PropertyKey, Value)>, ParseError> {
  let mut properties = vec![];
  loop {
    if input.maybe_read_tag(end_tag) {
      break;
    }
    let key = match read_object(de, input, heap)? {
      Value::I32(int) => PropertyKey::I32(int),
      Value::U32(uint) => PropertyKey::U32(uint),
      Value::Double(double) => PropertyKey::Double(double),
      Value::String(str) => PropertyKey::String(str),
      value => {
        return Err(input.err(ParseErrorKind::InvalidPropertyKey(value)));
      }
    };
    let value = read_object(de, input, heap)?;
    properties.push((key, value));
  }
  let property_count = input.read_varint()?;
  if property_count != properties.len() as u32 {
    return Err(input.err(ParseErrorKind::InvalidPropertyCount {
      expected: property_count,
      actual: properties.len() as u32,
    }));
  }
  Ok(properties)
}

fn read_js_object(
  de: &mut ValueDeserializer,
  input: &mut Input<'_>,
  heap: &mut HeapBuilder,
) -> Result<Object, ParseError> {
  let properties =
    read_js_object_properties(de, input, heap, SerializationTag::EndJsObject)?;
  Ok(Object { properties })
}

fn read_sparse_js_array(
  de: &mut ValueDeserializer,
  input: &mut Input<'_>,
  heap: &mut HeapBuilder,
) -> Result<SparseArray, ParseError> {
  let length = input.read_varint()?;
  let properties = read_js_object_properties(
    de,
    input,
    heap,
    SerializationTag::EndSparseJsArray,
  )?;
  let expected_length = input.read_varint()?;
  if expected_length != length {
    return Err(input.err(ParseErrorKind::InvalidArrayElementsLength {
      expected: expected_length,
      actual: length,
    }));
  }
  Ok(SparseArray { length, properties })
}

fn read_dense_js_array(
  de: &mut ValueDeserializer,
  input: &mut Input<'_>,
  heap: &mut HeapBuilder,
) -> Result<DenseArray, ParseError> {
  let length = input.read_varint()?;
  input.ensure_minimum_available(length as usize)?;
  // This allocation is not unbounded, because it will only occur if the input
  // contained at least length bytes. This is checked by the function above.
  let mut elements = Vec::with_capacity(length as usize);
  for _ in 0..length {
    if input.maybe_read_tag(SerializationTag::TheHole) {
      elements.push(None);
    } else {
      let value = read_object(de, input, heap)?;
      elements.push(Some(value));
    }
  }
  let properties = read_js_object_properties(
    de,
    input,
    heap,
    SerializationTag::EndDenseJsArray,
  )?;
  let final_elements_length = input.read_varint()?;
  if final_elements_length != length {
    return Err(input.err(ParseErrorKind::InvalidArrayElementsLength {
      expected: final_elements_length,
      actual: length,
    }));
  }
  Ok(DenseArray {
    elements,
    properties,
  })
}

fn read_date(input: &mut Input<'_>) -> Result<Date, ParseError> {
  let time_since_epoch = input.read_double()?;
  Ok(Date::new(time_since_epoch))
}

fn read_regexp(
  de: &mut ValueDeserializer,
  input: &mut Input<'_>,
) -> Result<RegExp, ParseError> {
  let pattern = read_string_value(de, input)?;
  let flags = input.read_varint()?;
  let flags = RegExpFlags::from_bits(flags)
    .ok_or_else(|| input.err(ParseErrorKind::InvalidRegExpFlags(flags)))?;
  if flags.contains(RegExpFlags::LINEAR) {
    return Err(input.err(ParseErrorKind::InvalidRegExpFlags(flags.bits())));
  }
  if flags.contains(RegExpFlags::UNICODE)
    && flags.contains(RegExpFlags::UNICODE_SETS)
  {
    return Err(input.err(ParseErrorKind::InvalidRegExpFlags(flags.bits())));
  }
  Ok(RegExp { pattern, flags })
}

fn read_js_map(
  de: &mut ValueDeserializer,
  input: &mut Input<'_>,
  heap: &mut HeapBuilder,
) -> Result<Map, ParseError> {
  let mut entries = vec![];

  loop {
    if input.maybe_read_tag(SerializationTag::EndJsMap) {
      break;
    }
    let key = read_object(de, input, heap)?;
    let value = read_object(de, input, heap)?;
    entries.push((key, value));
  }

  let expected_length = input.read_varint()?;
  let actual_length = (entries.len() * 2) as u32;
  if expected_length != actual_length {
    return Err(input.err(ParseErrorKind::InvalidEntryCount {
      expected: expected_length,
      actual: actual_length,
    }));
  }

  Ok(Map { entries })
}

fn read_js_set(
  de: &mut ValueDeserializer,
  input: &mut Input<'_>,
  heap: &mut HeapBuilder,
) -> Result<Set, ParseError> {
  let mut values = vec![];

  loop {
    if input.maybe_read_tag(SerializationTag::EndJsSet) {
      break;
    }
    let value = read_object(de, input, heap)?;
    values.push(value);
  }

  let expected_length = input.read_varint()?;
  if expected_length != values.len() as u32 {
    return Err(input.err(ParseErrorKind::InvalidEntryCount {
      expected: expected_length,
      actual: values.len() as u32,
    }));
  }

  Ok(Set { values })
}

fn read_js_array_buffer(
  input: &mut Input<'_>,
  is_resizable: bool,
) -> Result<ArrayBuffer, ParseError> {
  let byte_length = input.read_varint()?;
  let mut max_byte_length = None;
  if is_resizable {
    let max_byte_length_value = input.read_varint()?;
    if max_byte_length_value < byte_length {
      return Err(input.err(
        ParseErrorKind::InvalidResizableArrayBufferMaxLength {
          byte_length,
          max_byte_length: max_byte_length_value,
        },
      ));
    }
    max_byte_length = Some(max_byte_length_value);
  }
  let bytes = input.read_bytes(byte_length as usize)?;
  // This allocation is not unbounded, because it will only occur if the input
  // contained at least byte_length bytes.
  let mut data = alloc_aligned_u8_slice(byte_length as usize);
  unsafe {
    std::ptr::copy_nonoverlapping(
      bytes.as_ptr(),
      data.as_mut_ptr(),
      byte_length as usize,
    )
  };
  Ok(ArrayBuffer {
    data,
    max_byte_length,
  })
}

fn read_js_array_buffer_view(
  input: &mut Input<'_>,
  buffer_byte_length: u32,
  buffer: HeapReference,
) -> Result<ArrayBufferView, ParseError> {
  let tag = input.read_varint_u8()?;
  let byte_offset = input.read_varint()?;
  let byte_length = input.read_varint()?;
  let flags = input.read_varint()?;

  if byte_offset > buffer_byte_length {
    return Err(input.err(ParseErrorKind::InvalidArrayBufferViewOffset {
      byte_offset,
      buffer_byte_length,
    }));
  }
  if byte_length > buffer_byte_length - byte_offset {
    return Err(input.err(ParseErrorKind::InvalidArrayBufferViewLength {
      byte_length,
      byte_offset,
      buffer_byte_length,
    }));
  }

  let is_length_tracking = flags & 0b1 != 0;
  let is_backed_by_rab = flags & 0b10 != 0;
  let kind;
  let element_size;
  if tag == ArrayBufferViewTag::DataView as u8 {
    kind = ArrayBufferViewKind::DataView;
    element_size = 1;
  } else if tag == ArrayBufferViewTag::Int8Array as u8 {
    kind = ArrayBufferViewKind::Int8Array;
    element_size = 1;
  } else if tag == ArrayBufferViewTag::Uint8Array as u8 {
    kind = ArrayBufferViewKind::Uint8Array;
    element_size = 1;
  } else if tag == ArrayBufferViewTag::Uint8ClampedArray as u8 {
    kind = ArrayBufferViewKind::Uint8ClampedArray;
    element_size = 1;
  } else if tag == ArrayBufferViewTag::Int16Array as u8 {
    kind = ArrayBufferViewKind::Int16Array;
    element_size = 2;
  } else if tag == ArrayBufferViewTag::Uint16Array as u8 {
    kind = ArrayBufferViewKind::Uint16Array;
    element_size = 2;
  } else if tag == ArrayBufferViewTag::Int32Array as u8 {
    kind = ArrayBufferViewKind::Int32Array;
    element_size = 4;
  } else if tag == ArrayBufferViewTag::Uint32Array as u8 {
    kind = ArrayBufferViewKind::Uint32Array;
    element_size = 4;
  } else if tag == ArrayBufferViewTag::Float32Array as u8 {
    kind = ArrayBufferViewKind::Float32Array;
    element_size = 4;
  } else if tag == ArrayBufferViewTag::Float64Array as u8 {
    kind = ArrayBufferViewKind::Float64Array;
    element_size = 8;
  } else if tag == ArrayBufferViewTag::BigInt64Array as u8 {
    kind = ArrayBufferViewKind::BigInt64Array;
    element_size = 8;
  } else if tag == ArrayBufferViewTag::BigUint64Array as u8 {
    kind = ArrayBufferViewKind::BigUint64Array;
    element_size = 8;
  } else {
    return Err(input.err(ParseErrorKind::InvalidArrayBufferViewTag(tag)));
  };

  if byte_offset % element_size != 0 {
    return Err(input.err(ParseErrorKind::UnalignedArrayBufferViewOffset {
      byte_offset,
      element_size,
    }));
  }
  if byte_length % element_size != 0 {
    return Err(input.err(ParseErrorKind::UnalignedArrayBufferViewLength {
      byte_length,
      element_size,
    }));
  }

  let length = byte_length / element_size;

  Ok(ArrayBufferView {
    kind,
    buffer,
    byte_offset,
    length,
    is_length_tracking,
    is_backed_by_rab,
  })
}

fn alloc_aligned_u8_slice(size: usize) -> Box<[u8]> {
  let layout = Layout::from_size_align(size, align_of::<u64>()).unwrap();
  let ptr = unsafe { std::alloc::alloc_zeroed(layout) };
  let type_vec = unsafe { Vec::<u8>::from_raw_parts(ptr, size, size) };
  debug_assert_eq!(type_vec.as_ptr() as usize % align_of::<u64>(), 0);
  debug_assert_eq!(type_vec.as_ptr() as usize, ptr as usize);
  type_vec.into_boxed_slice()
}

fn read_transferred_js_array_buffer(
  de: &mut ValueDeserializer,
  input: &mut Input<'_>,
) -> Result<ArrayBuffer, ParseError> {
  let id = input.read_varint()?;
  let Some(ab) = de.transfer_map.remove(&id) else {
    return Err(input.err(ParseErrorKind::MissingTransferredArrayBuffer(id)));
  };
  Ok(ab)
}

fn read_js_error(
  de: &mut ValueDeserializer,
  input: &mut Input<'_>,
  heap: &mut HeapBuilder,
) -> Result<Error, ParseError> {
  let mut name = ErrorName::Error;
  let mut message = None;
  let mut cause = None;
  let mut stack = None;
  loop {
    let tag = input.read_varint_u8()?;
    if tag == ErrorTag::EvalErrorPrototype as u8 {
      name = ErrorName::EvalError;
    } else if tag == ErrorTag::RangeErrorPrototype as u8 {
      name = ErrorName::RangeError;
    } else if tag == ErrorTag::ReferenceErrorPrototype as u8 {
      name = ErrorName::ReferenceError;
    } else if tag == ErrorTag::SyntaxErrorPrototype as u8 {
      name = ErrorName::SyntaxError;
    } else if tag == ErrorTag::TypeErrorPrototype as u8 {
      name = ErrorName::TypeError;
    } else if tag == ErrorTag::UriErrorPrototype as u8 {
      name = ErrorName::UriError;
    } else if tag == ErrorTag::Message as u8 {
      let str = read_string_value(de, input)?;
      message = Some(str);
    } else if tag == ErrorTag::Stack as u8 {
      let str = read_string_value(de, input)?;
      stack = Some(str);
    } else if tag == ErrorTag::Cause as u8 {
      let obj = read_object(de, input, heap)?;
      cause = Some(obj);
    } else if tag == ErrorTag::End as u8 {
      break;
    } else {
      return Err(input.err(ParseErrorKind::UnexpectedErrorTag(tag)));
    }
  }

  Ok(Error {
    name,
    message,
    stack,
    cause,
  })
}

impl<'a> Input<'a> {
  /// Creates a new ParseError at the position that is currently being read.
  fn err_current(&self, kind: ParseErrorKind) -> ParseError {
    ParseError {
      position: self.position,
      kind,
    }
  }

  /// Creates a new ParseError at the position most recently read from.
  fn err(&self, kind: ParseErrorKind) -> ParseError {
    ParseError {
      position: self.position - 1,
      kind,
    }
  }

  fn expect_eof(&self) -> Result<(), ParseError> {
    if self.bytes.len() < self.position {
      return Err(self.err_current(ParseErrorKind::ExpectedEof));
    }
    Ok(())
  }

  fn ensure_minimum_available(&self, bytes: usize) -> Result<(), ParseError> {
    let available = self.bytes.len() - self.position;
    if available < bytes {
      return Err(
        self
          .err_current(ParseErrorKind::ExpectedMinimumBytes(bytes, available)),
      );
    }
    Ok(())
  }

  fn skip_padding(&mut self) {
    while self.bytes.get(self.position)
      == Some(&(SerializationTag::Padding as u8))
    {
      self.position += 1;
    }
  }

  fn read_byte(&mut self) -> Result<u8, ParseError> {
    let val = self
      .bytes
      .get(self.position)
      .ok_or_else(|| self.err_current(ParseErrorKind::UnexpectedEof))?;
    self.position += 1;
    Ok(*val)
  }

  fn read_bytes(&mut self, len: usize) -> Result<&[u8], ParseError> {
    let val = self
      .bytes
      .get(self.position..self.position + len)
      .ok_or_else(|| self.err_current(ParseErrorKind::UnexpectedEof))?;
    self.position += len;
    Ok(val)
  }

  fn read_bytes_copied<const N: usize>(
    &mut self,
  ) -> Result<[u8; N], ParseError> {
    let bytes = self.read_bytes(N)?;
    let mut val = [0u8; N];
    val.copy_from_slice(bytes);
    Ok(val)
  }

  fn expect_tag(&mut self, tag: SerializationTag) -> Result<(), ParseError> {
    self.skip_padding();
    let byte = self.read_byte()?;
    if byte != tag as u8 {
      return Err(self.err(ParseErrorKind::ExpectedTag(tag, byte)));
    }
    Ok(())
  }

  fn maybe_read_tag(&mut self, tag: SerializationTag) -> bool {
    self.skip_padding();
    match self.bytes.get(self.position) {
      Some(byte) if *byte == tag as u8 => {
        self.position += 1;
        true
      }
      _ => false,
    }
  }

  fn read_varint(&mut self) -> Result<u32, ParseError> {
    let mut value = 0u32;
    let mut i = 0;
    loop {
      let byte = self.read_byte()?;
      value |= ((byte & 0b01111111) as u32) << (i * 7);
      i += 1;
      if byte & 0b10000000 == 0 || i > size_of::<u32>() {
        break;
      }
    }
    Ok(value)
  }

  fn read_varint_u8(&mut self) -> Result<u8, ParseError> {
    let mut value = 0u8;
    let mut i = 0;
    loop {
      let byte = self.read_byte()?;
      value |= (byte & 0b01111111) << (i * 7);
      i += 1;
      if byte & 0b10000000 == 0 || i > size_of::<u8>() {
        break;
      }
    }
    Ok(value)
  }

  fn read_zigzag(&mut self) -> Result<i32, ParseError> {
    let unsigned = self.read_varint()?;
    Ok((unsigned >> 1) as i32 ^ -((unsigned & 1) as i32))
  }

  fn read_double(&mut self) -> Result<f64, ParseError> {
    let bytes = self.read_bytes_copied::<8>()?;
    Ok(f64::from_le_bytes(bytes))
  }
}
