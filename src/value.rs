use std::borrow::Cow;
use std::fmt::Debug;

use num_bigint::BigInt;
use rand::Rng;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum Value {
  Undefined,
  Null,
  Bool(bool),
  I32(i32),
  U32(u32),
  Double(f64),
  BigInt(BigInt),
  String(StringValue),
  HeapReference(HeapReference),
}

#[derive(Clone)]
pub enum StringValue {
  Wtf8(Wtf8String),
  OneByte(OneByteString),
  TwoByte(TwoByteString),
}

impl std::fmt::Debug for StringValue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Wtf8(s) => {
        write!(f, "Wtf8(\"")?;
        write!(f, "{}", s.as_str().escape_default())?;
        write!(f, "\")")
      }
      Self::OneByte(s) => {
        write!(f, "OneByte(\"")?;
        write!(f, "{}", s.as_str().escape_default())?;
        write!(f, "\")")
      }
      Self::TwoByte(s) => {
        write!(f, "TwoByte(\"")?;
        s.display_escaped(f)?;
        write!(f, "\")")
      }
    }
  }
}

impl StringValue {
  /// Create a new StringValue from a String. If the string is valid Latin-1 it
  /// will be stored as a OneByte string, otherwise it will be stored as a Utf8
  /// string.
  pub fn new(s: String) -> Self {
    if s.is_ascii() {
      Self::OneByte(OneByteString {
        bytes: s.into_bytes(),
        is_ascii: true, // We just checked that the string is ASCII.
      })
    } else {
      Self::Wtf8(Wtf8String {
        bytes: s.into_bytes(),
        is_utf8: true, // This string is valid UTF-8, because it was a String.
      })
    }
  }

  /// Turn the StringValue into a String.
  pub fn into_string(self) -> String {
    match self {
      Self::Wtf8(s) => s.into_string(),
      Self::OneByte(s) => s.into_string(),
      Self::TwoByte(chars) => chars.to_string(),
    }
  }

  /// Get the value of the string as a String.
  pub fn to_string(&self) -> Cow<'_, str> {
    match &self {
      Self::Wtf8(s) => s.as_str(),
      Self::OneByte(s) => s.as_str(),
      Self::TwoByte(chars) => Cow::Owned(chars.to_string()),
    }
  }
}

#[derive(Debug, Clone)]
pub struct Wtf8String {
  bytes: Vec<u8>,
  is_utf8: bool,
}

impl Wtf8String {
  /// Create a new Wtf8String from a Vec<u8>.
  pub fn new(bytes: Vec<u8>) -> Self {
    let is_utf8 = std::str::from_utf8(&bytes).is_ok();
    Self { bytes, is_utf8 }
  }

  /// Get the underlying bytes of this Wtf8String.
  pub fn as_bytes(&self) -> &[u8] {
    &self.bytes
  }

  /// Turn the Wtf8String into the underlying Vec<u8>.
  pub fn into_bytes(self) -> Vec<u8> {
    self.bytes
  }

  /// Turn this Wtf8String into a String.
  pub fn into_string(self) -> String {
    if self.is_utf8 {
      // SAFETY: The bytes are valid UTF-8.
      unsafe { String::from_utf8_unchecked(self.bytes) }
    } else {
      String::from_utf8_lossy(&self.bytes).into_owned()
    }
  }

  /// Get the value of this Wtf8String as a &str. If the bytes are not valid
  /// UTF-8, lossy conversion will be used.
  pub fn as_str(&self) -> Cow<'_, str> {
    if self.is_utf8 {
      // SAFETY: The bytes are valid UTF-8.
      let str = unsafe { std::str::from_utf8_unchecked(&self.bytes) };
      Cow::Borrowed(str)
    } else {
      String::from_utf8_lossy(&self.bytes)
    }
  }
}

#[derive(Debug, Clone)]
pub struct OneByteString {
  bytes: Vec<u8>,
  is_ascii: bool,
}

impl OneByteString {
  /// Create a new OneByteString from a Vec<u8>.
  pub fn new(bytes: Vec<u8>) -> Self {
    let is_ascii = bytes.is_ascii();
    Self { bytes, is_ascii }
  }

  /// Get the underlying bytes of this OneByteString.
  pub fn as_bytes(&self) -> &[u8] {
    &self.bytes
  }

  /// Turn the OneByteString into the underlying Vec<u8>.
  pub fn into_bytes(self) -> Vec<u8> {
    self.bytes
  }

  /// Turn this OneByteString into a String.
  pub fn into_string(self) -> String {
    if self.is_ascii {
      // SAFETY: The bytes are valid ASCII, which is a subset of UTF-8.
      unsafe { String::from_utf8_unchecked(self.bytes) }
    } else {
      // The string is latin1, so we have to convert it to UTF-8. WINDOWS_1252
      // is the same as latin1.
      let (str, _) =
        encoding_rs::WINDOWS_1252.decode_without_bom_handling(&self.bytes);
      match str {
        Cow::Borrowed(_) => {
          // SAFETY: The bytes are valid ASCII, which is a subset of UTF-8.
          unsafe { String::from_utf8_unchecked(self.bytes) }
        }
        Cow::Owned(string) => string,
      }
    }
  }

  /// Get the value of this OneByteString as a &str. This operation i
  /// infallible, as the bytes are guaranteed to be valid ASCII.
  pub fn as_str(&self) -> Cow<'_, str> {
    if self.is_ascii {
      // SAFETY: The bytes are valid ASCII, which is a subset of UTF-8.
      let str = unsafe { std::str::from_utf8_unchecked(&self.bytes) };
      Cow::Borrowed(str)
    } else {
      let (str, _) =
        encoding_rs::WINDOWS_1252.decode_without_bom_handling(&self.bytes);
      str
    }
  }
}

#[derive(Debug, Clone)]
pub struct TwoByteString(Vec<u16>);

impl TwoByteString {
  /// Create a new TwoByteString from a Vec<u16>. This operation is infallible,
  /// as the bytes are not checked for validity.
  pub fn new(chars: Vec<u16>) -> Self {
    Self(chars)
  }

  /// Get the underlying bytes of this TwoByteString.
  pub fn as_bytes(&self) -> &[u16] {
    &self.0
  }

  /// Turn the TwoByteString into the underlying Vec<u8>.
  pub fn into_bytes(self) -> Vec<u16> {
    self.0
  }

  /// Turn this TwoByteString into a String. If the bytes are not valid UTF-16,
  /// this operation will turn them into a String lossily.
  pub fn to_string(&self) -> String {
    String::from_utf16_lossy(&self.0)
  }

  /// Display the contents of the string in UTF-8, escaping any bytes that are
  /// not valid code points with a Unicode escape sequence (e.g. `\u{1234}`).
  /// Also escapes all ASCII control characters, and the characters `\` and `"`.
  pub fn display_escaped(
    &self,
    writer: &mut impl std::fmt::Write,
  ) -> std::fmt::Result {
    for res in std::char::decode_utf16(self.as_bytes().into_iter().map(|f| *f))
    {
      match res {
        Ok(char) => match char {
          '"' | '\\' => write!(writer, "\\{}", char)?,
          c if c.is_ascii_control() => {
            write!(writer, "{}", c.escape_unicode())?
          }
          _ => write!(writer, "{}", char)?,
        },
        Err(err) => write!(writer, "\\u{{{:x}}}", err.unpaired_surrogate())?,
      }
    }
    Ok(())
  }
}

pub enum HeapValue {
  /// new Boolean(bool)
  BooleanObject(bool),
  /// new Number(double)
  NumberObject(f64),
  /// Object(bigint)
  BigIntObject(BigInt),
  /// new String(string)
  StringObject(StringValue),
  /// new RegExp(pattern, flags)
  RegExp(RegExp),
  /// new Date(timeSinceEpoch)
  Date(Date),
  // { [key]: value }
  Object(Object),
  /// new Array(0)
  ///   .length = length
  ///   [properties.key] = properties.value
  /// and additional properties of the array object.
  SparseArray(SparseArray),
  /// new Array(...elements)
  ///   [properties.key] = properties.value
  DenseArray(DenseArray),
  /// new Map(...entries)
  Map(Map),
  /// new Set(...values)
  Set(Set),
  /// new ArrayBuffer(byteLength)
  ArrayBuffer(ArrayBuffer),
  /// new Uint8Array(buffer, byteOffset, length)
  /// new Uint8ClampedArray(buffer, byteOffset, length)
  /// new Int8Array(buffer, byteOffset, length)
  /// new Uint16Array(buffer, byteOffset, length)
  /// new Int16Array(buffer, byteOffset, length)
  /// new Uint32Array(buffer, byteOffset, length)
  /// new Int32Array(buffer, byteOffset, length)
  /// new Float32Array(buffer, byteOffset, length)
  /// new Float64Array(buffer, byteOffset, length)
  /// new BigInt64Array(buffer, byteOffset, length)
  /// new BigUint64Array(buffer, byteOffset, length)
  /// new DataView(buffer, byteOffset, byteLength)
  ArrayBufferView(ArrayBufferView),
  /// new Error(message, { cause: "foo" })
  Error(Error),
}

impl std::fmt::Debug for HeapValue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::BooleanObject(val) => {
        f.debug_tuple("BooleanObject").field(val).finish()
      }
      Self::NumberObject(val) => {
        f.debug_tuple("NumberObject").field(val).finish()
      }
      Self::BigIntObject(val) => {
        f.debug_tuple("BigIntObject").field(val).finish()
      }
      Self::StringObject(val) => {
        f.debug_tuple("StringObject").field(val).finish()
      }
      Self::RegExp(regexp) => std::fmt::Debug::fmt(regexp, f),
      Self::Date(date) => std::fmt::Debug::fmt(date, f),
      Self::Object(obj) => std::fmt::Debug::fmt(obj, f),
      Self::SparseArray(arr) => std::fmt::Debug::fmt(arr, f),
      Self::DenseArray(arr) => std::fmt::Debug::fmt(arr, f),
      Self::Map(map) => std::fmt::Debug::fmt(map, f),
      Self::Set(set) => std::fmt::Debug::fmt(set, f),
      Self::ArrayBuffer(ab) => std::fmt::Debug::fmt(ab, f),
      Self::ArrayBufferView(view) => std::fmt::Debug::fmt(view, f),
      Self::Error(err) => std::fmt::Debug::fmt(err, f),
    }
  }
}

#[derive(Clone)]
pub enum PropertyKey {
  I32(i32),
  U32(u32),
  String(StringValue),
}

impl std::fmt::Debug for PropertyKey {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::I32(index) => std::fmt::Debug::fmt(index, f),
      Self::U32(index) => std::fmt::Debug::fmt(index, f),
      Self::String(key) => std::fmt::Debug::fmt(key, f),
    }
  }
}

pub struct Object {
  pub properties: Vec<(PropertyKey, Value)>,
}

impl std::fmt::Debug for Object {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Object ")?;
    let mut map = f.debug_map();
    for (key, value) in &self.properties {
      map.entry(key, value);
    }
    map.finish()
  }
}

pub struct DenseArray {
  /// The elements of the array. The length of this vector is the length of the
  /// array. If an element is None, it is the same as if the array had a hole
  /// there.
  pub elements: Vec<Option<Value>>,
  /// Additional properties of the array object.
  pub properties: Vec<(PropertyKey, Value)>,
}

struct Hole;

impl Debug for Hole {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "/* hole */")
  }
}

impl std::fmt::Debug for DenseArray {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "DenseArray ")?;
    let mut list = f.debug_list();
    for value in &self.elements {
      if let Some(value) = value {
        list.entry(value);
      } else {
        list.entry(&Hole);
      }
    }
    list.finish()?;
    write!(f, " ")?;
    let mut map = f.debug_map();
    for (key, value) in &self.properties {
      map.entry(key, value);
    }
    map.finish()?;
    Ok(())
  }
}

pub struct SparseArray {
  pub length: u32,
  pub properties: Vec<(PropertyKey, Value)>,
}

impl std::fmt::Debug for SparseArray {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "SparseArray({}) ", self.length)?;
    let mut map = f.debug_map();
    for (key, value) in &self.properties {
      map.entry(key, value);
    }
    map.finish()
  }
}

#[derive(Debug)]
pub struct RegExp {
  pub pattern: StringValue,
  pub flags: u32,
}

#[derive(Debug)]
pub struct Date {
  // The time since the epoch in milliseconds. This is a double, but it is
  // always a whole number or NaN, and it is never infinite.
  time_since_epoch: f64,
}

fn double_to_integer(x: f64) -> f64 {
  if x.is_nan() || x == 0.0 {
    return x;
  }
  if !x.is_finite() {
    return x;
  }
  let x = if x > 0.0 { x.floor() } else { x.ceil() };
  x + 0.0 // ensure that -0.0 is normalized to 0.0
}

impl Date {
  pub fn new(time_since_epoch: f64) -> Date {
    const MAX_TIME_IN_MS: f64 = (864_000_000i64 * 10_000_000i64) as f64;
    if time_since_epoch < -MAX_TIME_IN_MS || time_since_epoch > MAX_TIME_IN_MS {
      let time_since_epoch = double_to_integer(time_since_epoch);
      Date { time_since_epoch }
    } else {
      Date {
        time_since_epoch: f64::NAN,
      }
    }
  }

  pub fn ms_since_epoch(&self) -> Option<i64> {
    if self.time_since_epoch.is_nan() {
      return None;
    }
    // Safety: We checked that the value is not NaN above, and we check in the
    // constructor that the value is a whole number and not infinite.
    Some(unsafe { self.time_since_epoch.to_int_unchecked() })
  }
}

pub struct Map {
  pub entries: Vec<(Value, Value)>,
}

impl std::fmt::Debug for Map {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Map ")?;
    let mut map = f.debug_map();
    for (key, value) in &self.entries {
      map.entry(key, value);
    }
    map.finish()
  }
}

pub struct Set {
  pub values: Vec<Value>,
}

impl std::fmt::Debug for Set {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Set ")?;
    let mut list = f.debug_set();
    for value in &self.values {
      list.entry(value);
    }
    list.finish()
  }
}

#[derive(Debug)]
pub struct ArrayBuffer {
  /// The raw bytes of the buffer. We ensure that this is always aligned to 8
  /// bytes, so that we can cast it to a [[u64] / [i64]] when appropriate.
  pub(crate) data: Box<[u8]>,
  /// The maximum byte length of the buffer. If this is None, resizability is
  /// disabled. If this is Some(n), then the buffer can be resized to a maximum
  /// of n bytes.
  pub max_byte_length: Option<u32>,
}

impl ArrayBuffer {
  pub fn byte_length(&self) -> u32 {
    self.data.len() as u32
  }
}

#[derive(Debug)]
pub enum ArrayBufferViewKind {
  Int8Array,
  Uint8Array,
  Uint8ClampedArray,
  Int16Array,
  Uint16Array,
  Int32Array,
  Uint32Array,
  Float32Array,
  Float64Array,
  BigInt64Array,
  BigUint64Array,
  DataView,
}

#[derive(Debug)]
pub struct ArrayBufferView {
  pub kind: ArrayBufferViewKind,
  pub buffer: HeapReference,
  pub byte_offset: u32,
  pub length: u32,
  pub is_length_tracking: bool,
  pub is_backed_by_rab: bool,
}

#[derive(Debug)]
pub enum ErrorKind {
  Error,
  EvalError,
  RangeError,
  ReferenceError,
  SyntaxError,
  TypeError,
  UriError,
}

#[derive(Debug)]
pub struct Error {
  pub kind: ErrorKind,
  pub message: Option<StringValue>,
  pub stack: Option<StringValue>,
  pub cause: Option<Value>,
}

pub struct HeapBuilder {
  heap_id: usize,
  values: Vec<Option<HeapValue>>,
}

impl Default for HeapBuilder {
  fn default() -> Self {
    Self {
      heap_id: rand::thread_rng().gen(),
      values: vec![],
    }
  }
}

#[derive(Error, Debug)]
pub struct HeapBuildError {
  index: usize,
}

impl std::fmt::Display for HeapBuildError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "reference {} for heap builder has not been populated yet",
      self.index
    )
  }
}

impl HeapBuilder {
  pub fn reserve(&mut self) -> HeapReference {
    let index = self.values.len();
    self.values.push(None);
    HeapReference {
      heap_id: self.heap_id,
      index,
    }
  }

  pub fn insert_reserved(
    &mut self,
    reference: HeapReference,
    value: HeapValue,
  ) {
    assert!(reference.heap_id == self.heap_id);
    assert!(
      self.values[reference.index].replace(value).is_none(),
      "reference {} for heap builder has already been populated",
      reference.index
    );
  }

  pub fn insert(&mut self, value: HeapValue) -> HeapReference {
    let reference = self.reserve();
    self.insert_reserved(reference, value);
    reference
  }

  pub fn try_open(&self, reference: HeapReference) -> Option<&HeapValue> {
    assert!(reference.heap_id == self.heap_id);
    self.values[reference.index].as_ref()
  }

  pub(crate) fn reference_by_id(&mut self, id: u32) -> Option<HeapReference> {
    let index = id as usize;
    if self.values.len() <= index {
      return None;
    }
    Some(HeapReference {
      heap_id: self.heap_id,
      index,
    })
  }

  pub fn build(self) -> Result<Heap, HeapBuildError> {
    let mut map = Vec::with_capacity(self.values.len());
    for value in self.values {
      map.push(value.ok_or(HeapBuildError { index: map.len() })?);
    }
    Ok(Heap {
      heap_id: self.heap_id,
      values: map,
    })
  }
}

pub struct Heap {
  heap_id: usize,
  values: Vec<HeapValue>,
}

impl std::fmt::Debug for Heap {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Heap ")?;
    let mut map = f.debug_map();
    for (index, value) in self.values.iter().enumerate() {
      map.entry(&index, value);
    }
    map.finish()
  }
}

impl Heap {
  pub fn is_empty(&self) -> bool {
    self.values.is_empty()
  }

  pub fn insert(&mut self, value: HeapValue) -> HeapReference {
    let index = self.values.len();
    assert!(
      index <= u32::MAX as usize,
      "can not have more than u32::MAX HeapValues in Heap"
    );
    self.values.push(value);
    HeapReference {
      heap_id: self.heap_id,
      index,
    }
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct HeapReference {
  heap_id: usize,
  index: usize,
}

impl std::fmt::Debug for HeapReference {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "*{}", self.index)
  }
}

impl HeapReference {
  pub fn open<'a>(&self, heap: &'a Heap) -> &'a HeapValue {
    assert!(self.heap_id == heap.heap_id);
    &heap.values[self.index]
  }
}