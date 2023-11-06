use std::collections::HashMap;

use num_bigint::BigInt;
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
use crate::value::Map;
use crate::value::Object;
use crate::value::PropertyKey;
use crate::value::RegExp;
use crate::value::Set;
use crate::value::SparseArray;
use crate::Heap;
use crate::HeapReference;
use crate::HeapValue;
use crate::StringValue;
use crate::Value;

#[derive(Debug, Error)]
pub enum SerializationError {
  #[error("recursion depth limit exceeded")]
  RecursionDepthLimitExceeded,
  #[error("a dangling heap reference was encountered")]
  DanglingHeapReference,
  #[error("a string was too long to serialize")]
  StringTooLong,
  #[error("a BigInt was too large to serialize")]
  BigIntTooLarge,
  #[error("an object has too many properties to serialize")]
  TooManyObjectProperties,
  #[error("an array has too many elements to serialize")]
  ArrayTooLong,
  #[error("a map has too many entries to serialize")]
  MapTooLarge,
  #[error("a set has too many entries to serialize")]
  SetTooLarge,
}

#[derive(Default)]
pub struct ValueSerializer {
  data: Vec<u8>,
  id_map: HashMap<HeapReference, u32>,
  recursion_depth: usize,
}

const RECURSION_DEPTH_LIMIT: usize = 256;
const WIRE_FORMAT_VERSION: u32 = 15;

impl ValueSerializer {
  pub fn finish(
    mut self,
    heap: &Heap,
    value: &Value,
  ) -> Result<Vec<u8>, SerializationError> {
    self.write_header();
    self.write_value(heap, value)?;
    Ok(self.data)
  }

  fn write_header(&mut self) {
    self.write_tag(SerializationTag::Version);
    self.write_varint(WIRE_FORMAT_VERSION);
  }

  fn write_value(
    &mut self,
    heap: &Heap,
    value: &Value,
  ) -> Result<(), SerializationError> {
    match value {
      Value::Undefined => self.write_tag(SerializationTag::Undefined),
      Value::Null => self.write_tag(SerializationTag::Null),
      Value::Bool(true) => self.write_tag(SerializationTag::True),
      Value::Bool(false) => self.write_tag(SerializationTag::False),
      Value::I32(smi) => self.write_smi(*smi),
      Value::U32(int) => self.write_u32(int),
      Value::Double(double) => self.write_number(*double),
      Value::BigInt(bigint) => self.write_bigint(bigint)?,
      Value::String(str) => self.write_string(str)?,
      Value::HeapReference(reference) => {
        self.recursion_depth += 1;
        self.write_heap_reference(heap, *reference)?;
        self.recursion_depth -= 1;
      }
    };
    Ok(())
  }

  fn write_tag(&mut self, tag: SerializationTag) {
    self.data.push(tag as u8)
  }

  fn write_varint(&mut self, value: u32) {
    // Writes an unsigned integer as a base-128 varint.
    // The number is written, 7 bits at a time, from the least significant to the
    // most significant 7 bits. Each byte, except the last, has the MSB set.
    // See also https://developers.google.com/protocol-buffers/docs/encoding
    let mut value = value;
    while value >= 0x80 {
      self.data.push(((value & 0x7f) | 0x80) as u8);
      value >>= 7;
    }
    self.data.push(value as u8);
  }

  fn write_varint_u8(&mut self, value: u8) {
    self.write_varint(value as u32);
  }

  fn write_zigzag(&mut self, value: i32) {
    // Writes a signed integer as a varint using ZigZag encoding (i.e. 0 is
    // encoded as 0, -1 as 1, 1 as 2, -2 as 3, and so on).
    // See also https://developers.google.com/protocol-buffers/docs/encoding
    self.write_varint(
      ((value << 1) ^ (value >> (i32::BITS as usize - 1))) as u32,
    );
  }

  fn write_double(&mut self, value: f64) {
    self.data.extend_from_slice(&value.to_le_bytes());
  }

  fn write_smi(&mut self, val: i32) {
    self.write_tag(SerializationTag::Int32);
    self.write_zigzag(val);
  }

  fn write_u32(&mut self, int: &u32) {
    self.write_tag(SerializationTag::Uint32);
    self.write_varint(*int);
  }

  fn write_number(&mut self, val: f64) {
    self.write_tag(SerializationTag::Double);
    self.write_double(val);
  }

  fn write_bigint(&mut self, val: &BigInt) -> Result<(), SerializationError> {
    self.write_tag(SerializationTag::BigInt);
    self.write_bigint_contents(val)?;
    Ok(())
  }

  fn write_bigint_contents(
    &mut self,
    val: &BigInt,
  ) -> Result<(), SerializationError> {
    let (sign, bytes) = val.to_bytes_le();
    let mut bitfield = 0u32;
    if sign == num_bigint::Sign::Minus {
      bitfield |= 1;
    }
    let length: u32 = bytes
      .len()
      .try_into()
      .map_err(|_| SerializationError::BigIntTooLarge)?;
    if length > 0x7fff_ffff {
      return Err(SerializationError::BigIntTooLarge);
    }
    bitfield |= length << 1;
    self.write_varint(bitfield);
    self.data.extend_from_slice(&bytes);
    Ok(())
  }

  fn write_string(
    &mut self,
    str: &StringValue,
  ) -> Result<(), SerializationError> {
    match str {
      StringValue::Wtf8(wtf8) => {
        self.write_tag(SerializationTag::Utf8String);
        let bytes = wtf8.as_bytes();
        let length: u32 = bytes
          .len()
          .try_into()
          .map_err(|_| SerializationError::StringTooLong)?;
        self.write_varint(length);
        self.data.extend_from_slice(bytes);
      }
      StringValue::OneByte(str) => {
        self.write_tag(SerializationTag::OneByteString);
        let bytes = str.as_bytes();
        let length: u32 = str
          .as_bytes()
          .len()
          .try_into()
          .map_err(|_| SerializationError::StringTooLong)?;
        self.write_varint(length);
        self.data.extend_from_slice(bytes);
      }
      StringValue::TwoByte(str) => {
        let bytes = str.as_u8_bytes();
        let length: u32 = bytes
          .len()
          .try_into()
          .map_err(|_| SerializationError::StringTooLong)?;
        if (self.data.len() + 1 + bytes_needed_for_varint(length)) & 0x1 == 1 {
          self.write_tag(SerializationTag::Padding);
        }
        self.write_tag(SerializationTag::TwoByteString);
        self.write_varint(length);
        self.data.extend_from_slice(bytes);
      }
    }
    Ok(())
  }

  fn write_heap_reference(
    &mut self,
    heap: &Heap,
    reference: HeapReference,
  ) -> Result<(), SerializationError> {
    let Some(value) = reference.try_open(heap) else {
      return Err(SerializationError::DanglingHeapReference);
    };
    match value {
      HeapValue::ArrayBufferView(abv)
        if !self.id_map.contains_key(&reference) =>
      {
        self.recursion_depth += 1;
        self.write_heap_reference(heap, abv.buffer)?;
        self.recursion_depth -= 1;
        self.write_heap_value_inner(heap, reference, value)
      }
      _ => self.write_heap_value_inner(heap, reference, value),
    }
  }

  fn write_heap_value_inner(
    &mut self,
    heap: &Heap,
    reference: HeapReference,
    value: &HeapValue,
  ) -> Result<(), SerializationError> {
    let next_id: u32 = self.id_map.len() as u32;
    match self.id_map.entry(reference) {
      std::collections::hash_map::Entry::Occupied(entry) => {
        let id = *entry.get();
        self.write_tag(SerializationTag::ObjectReference);
        self.write_varint(id);
        return Ok(());
      }
      std::collections::hash_map::Entry::Vacant(entry) => {
        entry.insert(next_id);
        next_id
      }
    };

    if self.recursion_depth > RECURSION_DEPTH_LIMIT {
      return Err(SerializationError::RecursionDepthLimitExceeded);
    }

    match value {
      HeapValue::BooleanObject(true) => {
        self.write_tag(SerializationTag::TrueObject);
      }
      HeapValue::BooleanObject(false) => {
        self.write_tag(SerializationTag::FalseObject);
      }
      HeapValue::NumberObject(double) => {
        self.write_tag(SerializationTag::NumberObject);
        self.write_double(*double);
      }
      HeapValue::BigIntObject(bigint) => {
        self.write_tag(SerializationTag::BigIntObject);
        self.write_bigint_contents(bigint)?;
      }
      HeapValue::StringObject(str) => {
        self.write_tag(SerializationTag::StringObject);
        self.write_string(str)?;
      }
      HeapValue::RegExp(regexp) => self.write_regexp(regexp)?,
      HeapValue::Date(date) => {
        self.write_date(date);
      }
      HeapValue::Object(obj) => self.write_object(heap, obj)?,
      HeapValue::SparseArray(arr) => self.write_sparse_array(heap, arr)?,
      HeapValue::DenseArray(arr) => self.write_dense_array(heap, arr)?,
      HeapValue::Map(map) => self.write_map(heap, map)?,
      HeapValue::Set(set) => self.write_set(heap, set)?,
      HeapValue::ArrayBuffer(ab) => self.write_array_buffer(ab),
      HeapValue::ArrayBufferView(abv) => self.write_array_buffer_view(abv),
      HeapValue::Error(err) => self.write_error(heap, err)?,
    };
    Ok(())
  }

  fn write_regexp(
    &mut self,
    regexp: &RegExp,
  ) -> Result<(), SerializationError> {
    self.write_tag(SerializationTag::RegExp);
    self.write_string(&regexp.pattern)?;
    self.write_varint(regexp.flags.bits());
    Ok(())
  }

  fn write_date(&mut self, date: &Date) {
    self.write_tag(SerializationTag::Date);
    self.write_double(date.time_since_epoch);
  }

  fn write_object(
    &mut self,
    heap: &Heap,
    obj: &Object,
  ) -> Result<(), SerializationError> {
    self.write_tag(SerializationTag::BeginJsObject);
    self.write_object_properties(
      heap,
      &obj.properties,
      SerializationTag::EndJsObject,
    )?;
    Ok(())
  }

  fn write_object_properties(
    &mut self,
    heap: &Heap,
    properties: &[(PropertyKey, Value)],
    end_tag: SerializationTag,
  ) -> Result<(), SerializationError> {
    let property_count: u32 = properties
      .len()
      .try_into()
      .map_err(|_| SerializationError::TooManyObjectProperties)?;
    for (key, value) in properties {
      match key {
        PropertyKey::I32(smi) => self.write_smi(*smi),
        PropertyKey::U32(num) => self.write_u32(num),
        PropertyKey::Double(double) => self.write_number(*double),
        PropertyKey::String(str) => self.write_string(str)?,
      }
      self.write_value(heap, value)?;
    }
    self.write_tag(end_tag);
    self.write_varint(property_count);
    Ok(())
  }

  fn write_sparse_array(
    &mut self,
    heap: &Heap,
    arr: &SparseArray,
  ) -> Result<(), SerializationError> {
    self.write_tag(SerializationTag::BeginSparseJsArray);
    self.write_varint(arr.length);
    self.write_object_properties(
      heap,
      &arr.properties,
      SerializationTag::EndSparseJsArray,
    )?;
    self.write_varint(arr.length);
    Ok(())
  }

  fn write_dense_array(
    &mut self,
    heap: &Heap,
    arr: &DenseArray,
  ) -> Result<(), SerializationError> {
    self.write_tag(SerializationTag::BeginDenseJsArray);
    let length: u32 = arr
      .elements
      .len()
      .try_into()
      .map_err(|_| SerializationError::ArrayTooLong)?;
    self.write_varint(length);
    for value in &arr.elements {
      if let Some(value) = value {
        self.write_value(heap, value)?;
      } else {
        self.write_tag(SerializationTag::TheHole);
      }
    }
    self.write_object_properties(
      heap,
      &arr.properties,
      SerializationTag::EndDenseJsArray,
    )?;
    self.write_varint(length);
    Ok(())
  }

  fn write_map(
    &mut self,
    heap: &Heap,
    map: &Map,
  ) -> Result<(), SerializationError> {
    let size: u32 = map
      .entries
      .len()
      .try_into()
      .map_err(|_| SerializationError::MapTooLarge)?;
    let length = size.checked_mul(2).ok_or(SerializationError::MapTooLarge)?;
    self.write_tag(SerializationTag::BeginJsMap);
    for (key, value) in &map.entries {
      self.write_value(heap, key)?;
      self.write_value(heap, value)?;
    }
    self.write_tag(SerializationTag::EndJsMap);
    self.write_varint(length);
    Ok(())
  }

  fn write_set(
    &mut self,
    heap: &Heap,
    set: &Set,
  ) -> Result<(), SerializationError> {
    let size: u32 = set
      .values
      .len()
      .try_into()
      .map_err(|_| SerializationError::SetTooLarge)?;
    self.write_tag(SerializationTag::BeginJsSet);
    for value in &set.values {
      self.write_value(heap, value)?;
    }
    self.write_tag(SerializationTag::EndJsSet);
    self.write_varint(size);
    Ok(())
  }

  fn write_array_buffer(&mut self, ab: &ArrayBuffer) {
    if let Some(max_byte_length) = ab.max_byte_length {
      self.write_tag(SerializationTag::ResizableArrayBuffer);
      self.write_varint(ab.byte_length());
      self.write_varint(max_byte_length);
    } else {
      self.write_tag(SerializationTag::ArrayBuffer);
      self.write_varint(ab.byte_length());
    }
    self.data.extend_from_slice(ab.as_u8_slice());
  }

  fn write_array_buffer_view(&mut self, abv: &ArrayBufferView) {
    self.write_tag(SerializationTag::ArrayBufferView);
    let tag = match abv.kind {
      ArrayBufferViewKind::Int8Array => ArrayBufferViewTag::Int8Array,
      ArrayBufferViewKind::Uint8Array => ArrayBufferViewTag::Uint8Array,
      ArrayBufferViewKind::Uint8ClampedArray => {
        ArrayBufferViewTag::Uint8ClampedArray
      }
      ArrayBufferViewKind::Int16Array => ArrayBufferViewTag::Int16Array,
      ArrayBufferViewKind::Uint16Array => ArrayBufferViewTag::Uint16Array,
      ArrayBufferViewKind::Int32Array => ArrayBufferViewTag::Int32Array,
      ArrayBufferViewKind::Uint32Array => ArrayBufferViewTag::Uint32Array,
      ArrayBufferViewKind::Float32Array => ArrayBufferViewTag::Float32Array,
      ArrayBufferViewKind::Float64Array => ArrayBufferViewTag::Float64Array,
      ArrayBufferViewKind::BigInt64Array => ArrayBufferViewTag::BigInt64Array,
      ArrayBufferViewKind::BigUint64Array => ArrayBufferViewTag::BigUint64Array,
      ArrayBufferViewKind::DataView => ArrayBufferViewTag::DataView,
    };
    self.write_varint_u8(tag as u8);
    self.write_varint(abv.byte_offset);
    self.write_varint(abv.length * abv.kind.byte_width());
    let mut flags = 0u32;
    if abv.is_length_tracking {
      flags |= 0b1;
    }
    if abv.is_backed_by_rab {
      flags |= 0b10;
    }
    self.write_varint(flags);
  }

  fn write_error(
    &mut self,
    heap: &Heap,
    err: &Error,
  ) -> Result<(), SerializationError> {
    self.write_tag(SerializationTag::Error);
    let name_tag = match err.name {
      ErrorName::Error => None,
      ErrorName::EvalError => Some(ErrorTag::EvalErrorPrototype),
      ErrorName::RangeError => Some(ErrorTag::RangeErrorPrototype),
      ErrorName::ReferenceError => Some(ErrorTag::ReferenceErrorPrototype),
      ErrorName::SyntaxError => Some(ErrorTag::SyntaxErrorPrototype),
      ErrorName::TypeError => Some(ErrorTag::TypeErrorPrototype),
      ErrorName::UriError => Some(ErrorTag::UriErrorPrototype),
    };
    if let Some(tag) = name_tag {
      self.write_varint(tag as u32);
    }
    if let Some(message) = &err.message {
      self.write_varint(ErrorTag::Message as u32);
      self.write_string(message)?;
    }
    if let Some(cause) = &err.cause {
      self.write_varint(ErrorTag::Cause as u32);
      self.write_value(heap, cause)?;
    }
    if let Some(stack) = &err.stack {
      self.write_varint(ErrorTag::Stack as u32);
      self.write_string(stack)?;
    }

    self.write_varint(ErrorTag::End as u32);

    Ok(())
  }
}

fn bytes_needed_for_varint(value: u32) -> usize {
  let mut value = value;
  let mut bytes = 1;
  while value >= 0x80 {
    bytes += 1;
    value >>= 7;
  }
  bytes
}
