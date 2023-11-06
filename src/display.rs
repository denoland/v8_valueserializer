use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;
use std::f32;
use std::fmt::LowerHex;
use std::fmt::Write;

use crate::value::ArrayBuffer;
use crate::value::ArrayBufferView;
use crate::value::ArrayBufferViewKind;
use crate::value::PropertyKey;
use crate::Heap;
use crate::HeapReference;
use crate::HeapValue;
use crate::StringValue;
use crate::TwoByteString;
use crate::Value;

enum FollowUpTasks {
  PropertyAssignment {
    target: HeapReference,
    key: PropertyKey,
    value: Value,
    /// To ensure correct property ordering on objects, all properties for a
    /// given target that have this flag, must be set in the order in which they
    /// are in the follow up task list. If this task is ready to render, but
    /// there are other tasks for the same target that require ordering before
    /// this one, and they are not ready to render yet, then this task will be
    /// skipped anyway until the other tasks are rendered.
    requires_ordering: bool,
  },
  MapSet {
    target: HeapReference,
    key: Value,
    value: Value,
  },
  SetAdd {
    target: HeapReference,
    value: Value,
  },
  ArrayBufferSet {
    target: HeapReference,
    kind: ArrayBufferViewKind,
  },
}

struct Displayer<'h, W: Write> {
  heap: &'h Heap,
  writer: W,
  indent: usize,
  /// A map that keeps track of which heap objects have been seen, and what heap
  /// objects they reference and are referenced by. This is used to track cycles
  /// and determine whether an object can be inlined or not.
  deps: HashMap<HeapReference, HeapObjectInfo>,
  /// The order in which heap objects were seen.
  order: Vec<HeapReference>,
  /// For rendered objects, the name of the variable that they were assigned to.
  idents: HashMap<HeapReference, String>,
  /// Follow up rendering tasks that need to be done after a given object has
  /// been rendered and assigned to a variable.
  follow_up_tasks: Vec<FollowUpTasks>,
}

#[derive(Default, Debug)]
struct DependencyInfo {
  /// A map that keeps track of which heap objects have been seen, and what heap
  /// objects they reference and are referenced by. This is used to track cycles.
  objects: HashMap<HeapReference, HeapObjectInfo>,
  /// The order in which heap objects were seen.
  order: Vec<HeapReference>,
  /// The stack of objects that are currently being visited.
  stack: Vec<HeapReference>,
}

#[derive(Default, Debug)]
struct HeapObjectInfo {
  /// The heap objects that this heap object references.
  dependencies: HashSet<HeapReference>,
  /// The heap objects that reference this heap object.
  dependants: HashSet<HeapReference>,
  /// The count of dependants for this heap object. This is used to determine
  /// whether an object can be inlined or not. This can differ from the size of
  /// `dependants` because some dependants may reference the same heap object
  /// multiple times.
  dependants_count: usize,
  /// Whether this object needs to be assigned to a binding. This can happen for
  /// example for sparse arrays, or dense arrays with properties, or because
  /// this object is circularly referenced.
  requires_binding: bool,
}

impl HeapObjectInfo {
  /// Returns true if this object can be inlined. This is true if the object is
  /// only referenced by one other object, and it doesn't need to be assigned to
  /// a binding because it needs to be modified later (e.g. because it is a
  /// sparse array, or a dense array with properties, or because it is
  /// circularly referenced).
  fn inlineable(&self) -> bool {
    self.dependants_count < 2 && !self.requires_binding
  }
}

impl<'h, W: Write> Displayer<'h, W> {
  fn display(
    heap: &'h Heap,
    value: &Value,
    opts: DisplayOptions,
    writer: W,
  ) -> std::fmt::Result {
    let mut deps = DependencyInfo::default();

    macro_rules! visit_and_record {
      ($heap:expr, $deps:expr, $referrer:expr, $referred:expr) => {
        let referrer = $deps.objects.get_mut(&$referrer).unwrap();
        referrer.dependencies.insert($referred);
        visit($heap, $deps, $referred);
        let referred = $deps.objects.get_mut(&$referred).unwrap();
        referred.dependants.insert($referrer);
        referred.dependants_count += 1;
      };
    }

    fn visit(heap: &Heap, deps: &mut DependencyInfo, referrer: HeapReference) {
      match deps.objects.entry(referrer) {
        Entry::Occupied(mut entry) => {
          if deps.stack.contains(&referrer) {
            // We've found a cycle. Mark both the referrer and the referred as
            // requiring a binding.
            entry.get_mut().requires_binding = true;
            if let Some(referrer) = deps.stack.last() {
              let referrer = deps.objects.get_mut(referrer).unwrap();
              referrer.requires_binding = true;
            }
          }
          return;
        }
        Entry::Vacant(entry) => {
          entry.insert(HeapObjectInfo::default());
        }
      };
      deps.stack.push(referrer);

      let heap_value = referrer.open(heap);
      match heap_value {
        HeapValue::BooleanObject(_)
        | HeapValue::NumberObject(_)
        | HeapValue::BigIntObject(_)
        | HeapValue::StringObject(_)
        | HeapValue::RegExp(_)
        | HeapValue::Date(_) => {}
        HeapValue::Object(object) => {
          for (_, value) in &object.properties {
            if let Value::HeapReference(referred) = value {
              visit_and_record!(heap, deps, referrer, *referred);
            }
          }
        }
        HeapValue::SparseArray(arr) => {
          if !arr.properties.is_empty() {
            let referrer = deps.objects.get_mut(&referrer).unwrap();
            referrer.requires_binding = true;
          }
          for (_, value) in &arr.properties {
            if let Value::HeapReference(referred) = value {
              visit_and_record!(heap, deps, referrer, *referred);
            }
          }
        }
        HeapValue::DenseArray(arr) => {
          for value in arr.elements.iter().flatten() {
            if let Value::HeapReference(referred) = value {
              visit_and_record!(heap, deps, referrer, *referred);
            }
          }
          if !arr.properties.is_empty() {
            let referrer = deps.objects.get_mut(&referrer).unwrap();
            referrer.requires_binding = true;
          }
          for (_, value) in &arr.properties {
            if let Value::HeapReference(referred) = value {
              visit_and_record!(heap, deps, referrer, *referred);
            }
          }
        }
        HeapValue::Map(map) => {
          for (key, value) in &map.entries {
            if let Value::HeapReference(referred) = key {
              visit_and_record!(heap, deps, referrer, *referred);
            }
            if let Value::HeapReference(referred) = value {
              visit_and_record!(heap, deps, referrer, *referred);
            }
          }
        }
        HeapValue::Set(set) => {
          for value in &set.values {
            if let Value::HeapReference(referred) = value {
              visit_and_record!(heap, deps, referrer, *referred);
            }
          }
        }
        HeapValue::ArrayBuffer(ab) => {
          if ab.max_byte_length.is_some() {
            let referrer = deps.objects.get_mut(&referrer).unwrap();
            referrer.requires_binding = true;
          }
        }
        HeapValue::ArrayBufferView(view) => {
          visit_and_record!(heap, deps, referrer, view.buffer);
        }
        HeapValue::Error(err) => {
          if let Some(Value::HeapReference(referred)) = err.cause {
            visit_and_record!(heap, deps, referrer, referred);
          }
          let referrer = deps.objects.get_mut(&referrer).unwrap();
          referrer.requires_binding = true;
        }
      }

      deps.order.push(referrer);
      deps.stack.pop();
    }

    if let Value::HeapReference(reference) = value {
      visit(heap, &mut deps, *reference);
      let info = deps.objects.get_mut(reference).unwrap();
      info.dependants_count += 1; // the root object has one dependant that isn't in the stack (the displayer)
    }

    let multiline = deps.objects.values().any(|info| !info.inlineable());

    let mut this = Self {
      heap,
      writer,
      indent: 0,
      deps: deps.objects,
      order: deps.order,
      idents: HashMap::new(),
      follow_up_tasks: Vec::new(),
    };

    match opts.format {
      DisplayFormat::Expression if multiline => {
        writeln!(this.writer, "(function() {{")?;
        this.indent += 1;
      }
      DisplayFormat::Expression | DisplayFormat::Repl | DisplayFormat::Eval => {
      }
    }

    if let Value::HeapReference(reference) = value {
      assert_eq!(this.order.last(), Some(reference));

      for (i, mut reference) in this.order.clone().into_iter().enumerate() {
        let info = this.deps.get(&reference).unwrap();
        if !this.idents.contains_key(&reference) && !info.inlineable() {
          if let HeapValue::ArrayBuffer(ab) = reference.open(this.heap) {
            if let Some((
              backing_view_reference,
              _backing_view_info,
              _backing_view,
            )) = this.array_buffer_view_to_render_array_buffer_in(ab, info)
            {
              reference = backing_view_reference;
            };
          };

          // we need to assign this object to a variable because it is
          // referenced by multiple objects
          let ident = format!("v{}", i);
          this.display_indent(0)?;
          write!(this.writer, "const {} = ", ident)?;
          this.display_heap_value(reference.open(this.heap), &reference)?;
          writeln!(this.writer, ";")?;
          this.idents.insert(reference, ident.clone());

          // Run the follow up tasks for this object.
          let follow_up_tasks_to_run =
            std::mem::take(&mut this.follow_up_tasks);
          let mut deferred_targets_for_ordered_assignments = HashSet::new();
          for task in follow_up_tasks_to_run {
            match &task {
              FollowUpTasks::PropertyAssignment {
                target,
                key,
                value,
                requires_ordering,
              } => {
                let Some(ident) = this.idents.get(target).cloned() else {
                  this.follow_up_tasks.push(task);
                  continue;
                };
                if !this.is_ready_to_render(value)
                  || (*requires_ordering
                    && deferred_targets_for_ordered_assignments
                      .contains(target))
                {
                  if *requires_ordering {
                    deferred_targets_for_ordered_assignments.insert(*target);
                  }
                  this.follow_up_tasks.push(task);
                  continue;
                };
                this.display_indent(0)?;
                write!(this.writer, "{}[", ident)?;
                this.display_property_key(key)?;
                write!(this.writer, "] = ")?;
                this.display_value(value)?;
                writeln!(this.writer, ";")?;
              }
              FollowUpTasks::MapSet { target, key, value } => {
                let Some(ident) = this.idents.get(target).cloned() else {
                  this.follow_up_tasks.push(task);
                  continue;
                };
                if !this.is_ready_to_render(key)
                  || !this.is_ready_to_render(value)
                {
                  this.follow_up_tasks.push(task);
                  continue;
                }
                this.display_indent(0)?;
                write!(this.writer, "{}.set(", ident)?;
                this.display_value(key)?;
                write!(this.writer, ", ")?;
                this.display_value(value)?;
                writeln!(this.writer, ");")?;
              }
              FollowUpTasks::SetAdd { target, value } => {
                let Some(ident) = this.idents.get(target).cloned() else {
                  this.follow_up_tasks.push(task);
                  continue;
                };
                if !this.is_ready_to_render(value) {
                  this.follow_up_tasks.push(task);
                  continue;
                }
                this.display_indent(0)?;
                write!(this.writer, "{}.add(", ident)?;
                this.display_value(value)?;
                writeln!(this.writer, ");")?;
              }
              FollowUpTasks::ArrayBufferSet { target, kind } => {
                let Some(ident) = this.idents.get(target) else {
                  this.follow_up_tasks.push(task);
                  continue;
                };
                if !this.is_ready_to_render(value) {
                  this.follow_up_tasks.push(task);
                  continue;
                }
                write!(this.writer, "new {}({}).set(", kind, ident)?;
                let HeapValue::ArrayBuffer(buffer) = target.open(this.heap)
                else {
                  unreachable!()
                };
                this.display_array_buffer_data_array(*kind, buffer)?;
                writeln!(this.writer, ");")?;
              }
            }
          }
        }
      }
    };

    let mut return_has_parens = false;
    this.display_indent(0)?;
    match opts.format {
      DisplayFormat::Expression if multiline => write!(this.writer, "return ")?,
      DisplayFormat::Expression | DisplayFormat::Repl | DisplayFormat::Eval => {
        if let Value::HeapReference(reference) = value {
          let info = this.deps.get(reference).unwrap();
          if info.inlineable() {
            let value = reference.open(this.heap);
            if matches!(value, HeapValue::Object(..)) {
              return_has_parens = true;
              write!(this.writer, "(")?;
            }
          }
        }
      }
    }

    this.display_value(value)?;
    assert!(this.follow_up_tasks.is_empty());

    match opts.format {
      DisplayFormat::Expression if multiline => {
        write!(this.writer, "\n}})()")?;
        this.indent -= 1;
      }
      DisplayFormat::Expression | DisplayFormat::Repl | DisplayFormat::Eval => {
        if return_has_parens {
          write!(this.writer, ")")?;
        }
      }
    }

    Ok(())
  }

  fn is_ready_to_render(&self, value: &Value) -> bool {
    match value {
      Value::HeapReference(reference) => {
        let info = self.deps.get(reference).unwrap();
        if info.inlineable() || self.idents.get(reference).is_some() {
          return true;
        }
        match reference.open(self.heap) {
          HeapValue::ArrayBuffer(ab) => {
            if let Some((reference, info, _view)) =
              self.array_buffer_view_to_render_array_buffer_in(ab, info)
            {
              return info.inlineable()
                || self.idents.get(&reference).is_some();
            } else {
              false
            }
          }
          _ => false,
        }
      }
      _ => true,
    }
  }

  fn display_value(&mut self, value: &Value) -> std::fmt::Result {
    match value {
      Value::Undefined => write!(self.writer, "undefined"),
      Value::Null => write!(self.writer, "null"),
      Value::Bool(bool) => write!(self.writer, "{}", bool),
      Value::I32(val) => write!(self.writer, "{}", val),
      Value::U32(val) => write!(self.writer, "{}", val),
      Value::Double(val) => self.display_number(*val),
      Value::BigInt(val) => write!(self.writer, "{}n", val),
      Value::String(val) => self.display_string(val),
      Value::HeapReference(reference) => {
        if let Some(ident) = self.idents.get(reference) {
          write!(self.writer, "{}", ident)
        } else {
          let heap_value = reference.open(self.heap);
          self.display_heap_value(heap_value, reference)
        }
      }
    }
  }

  fn display_string(&mut self, string: &StringValue) -> std::fmt::Result {
    match string {
      StringValue::Wtf8(string) => {
        self.display_string_literal(string.as_str().chars())
      }
      StringValue::OneByte(string) => {
        self.display_string_literal(string.as_str().chars())
      }
      StringValue::TwoByte(string) => self.display_two_byte_str(string),
    }
  }

  fn display_string_literal(
    &mut self,
    chars: impl Iterator<Item = char>,
  ) -> std::fmt::Result {
    write!(self.writer, "\"")?;
    for char in chars {
      match char {
        '"' | '\\' => write!(self.writer, "\\{}", char)?,
        c if c.is_ascii_control() => {
          write!(self.writer, "{}", c.escape_unicode())?
        }
        _ => write!(self.writer, "{}", char)?,
      }
    }
    write!(self.writer, "\"")
  }

  fn display_two_byte_str(&mut self, str: &TwoByteString) -> std::fmt::Result {
    write!(self.writer, "\"")?;
    str.display_escaped(&mut self.writer)?;
    write!(self.writer, "\"")
  }

  fn display_number(&mut self, num: f64) -> std::fmt::Result {
    if num.is_nan() {
      write!(self.writer, "NaN")
    } else if num.is_infinite() {
      if num.is_sign_positive() {
        write!(self.writer, "Infinity")
      } else {
        write!(self.writer, "-Infinity")
      }
    } else {
      write!(self.writer, "{}", num)
    }
  }

  fn display_number_f32(&mut self, num: f32) -> std::fmt::Result {
    if num.is_nan() {
      write!(self.writer, "NaN")
    } else if num.is_infinite() {
      if num.is_sign_positive() {
        write!(self.writer, "Infinity")
      } else {
        write!(self.writer, "-Infinity")
      }
    } else {
      write!(self.writer, "{}", num)
    }
  }

  fn display_heap_value(
    &mut self,
    value: &HeapValue,
    reference: &HeapReference,
  ) -> std::fmt::Result {
    match value {
      HeapValue::BooleanObject(bool) => {
        write!(self.writer, "new Boolean({})", bool)?;
      }
      HeapValue::NumberObject(val) => {
        write!(self.writer, "new Number(")?;
        self.display_number(*val)?;
        write!(self.writer, ")")?;
      }
      HeapValue::BigIntObject(val) => {
        write!(self.writer, "new Object({}n)", val)?;
      }
      HeapValue::StringObject(str) => {
        write!(self.writer, "new String(")?;
        self.display_string(str)?;
        write!(self.writer, ")")?;
      }
      HeapValue::RegExp(regexp) => {
        write!(self.writer, "new RegExp(")?;
        self.display_string(&regexp.pattern)?;
        if !regexp.flags.is_empty() {
          write!(self.writer, ", ")?;
          self.display_string(&StringValue::new(regexp.flags.to_string()))?;
        }
        write!(self.writer, ")")?;
      }
      HeapValue::Date(date) => {
        write!(self.writer, "new Date(")?;
        if let Some(ms_since_epoch) = date.ms_since_epoch() {
          write!(self.writer, "{}", ms_since_epoch)?;
        } else {
          write!(self.writer, "NaN")?;
        }
        write!(self.writer, ")")?;
      }
      HeapValue::Object(object) => {
        writeln!(self.writer, "{{")?;
        for (key, value) in &object.properties {
          self.display_indent(1)?;
          self.display_property_key(key)?;
          write!(self.writer, ": ")?;
          self.indent += 1;
          if self.is_ready_to_render(value) {
            self.display_value(value)?;
          } else {
            writeln!(self.writer, "undefined /* circular */")?;
            self
              .follow_up_tasks
              .push(FollowUpTasks::PropertyAssignment {
                target: *reference,
                key: key.clone(),
                value: value.clone(),
                requires_ordering: false,
              });
          }
          writeln!(self.writer, ",")?;
          self.indent -= 1;
        }
        self.display_indent(0)?;
        write!(self.writer, "}}")?;
      }
      HeapValue::SparseArray(arr) => {
        write!(self.writer, "new Array({})", arr.length)?;
        if !arr.properties.is_empty() {
          for (key, value) in &arr.properties {
            self
              .follow_up_tasks
              .push(FollowUpTasks::PropertyAssignment {
                target: *reference,
                key: key.clone(),
                value: value.clone(),
                requires_ordering: true,
              });
          }
        }
      }
      HeapValue::DenseArray(arr) => {
        writeln!(self.writer, "[")?;
        for (i, value) in arr.elements.iter().enumerate() {
          self.display_indent(1)?;
          self.indent += 1;
          if let Some(value) = value {
            if self.is_ready_to_render(value) {
              self.display_value(value)?;
            } else {
              write!(self.writer, "undefined /* circular */")?;
              self
                .follow_up_tasks
                .push(FollowUpTasks::PropertyAssignment {
                  target: *reference,
                  key: PropertyKey::I32(i as i32),
                  value: value.clone(),
                  requires_ordering: false,
                });
            }
          } else {
            write!(self.writer, "/* hole */")?;
          }
          self.indent -= 1;
          writeln!(self.writer, ",")?;
        }
        self.display_indent(0)?;
        write!(self.writer, "]")?;
        for (key, value) in &arr.properties {
          self
            .follow_up_tasks
            .push(FollowUpTasks::PropertyAssignment {
              target: *reference,
              key: key.clone(),
              value: value.clone(),
              requires_ordering: true,
            });
        }
      }
      HeapValue::Map(map) => {
        if map.entries.is_empty() {
          writeln!(self.writer, "new Map()")?;
        } else {
          writeln!(self.writer, "new Map([")?;
          for (key, value) in &map.entries {
            if self.is_ready_to_render(key) || self.is_ready_to_render(value) {
              self.display_indent(1)?;
              write!(self.writer, "[")?;
              self.display_value(key)?;
              write!(self.writer, ", ")?;
              self.indent += 1;
              self.display_value(value)?;
              self.indent -= 1;
              writeln!(self.writer, "],")?;
            } else {
              self.follow_up_tasks.push(FollowUpTasks::MapSet {
                target: *reference,
                key: key.clone(),
                value: value.clone(),
              });
            }
          }
          self.display_indent(0)?;
          write!(self.writer, "])")?;
        }
      }
      HeapValue::Set(set) => {
        if set.values.is_empty() {
          writeln!(self.writer, "new Set()")?;
        } else {
          writeln!(self.writer, "new Set([")?;
          for value in &set.values {
            if self.is_ready_to_render(value) {
              self.display_indent(1)?;
              self.indent += 1;
              self.display_value(value)?;
              self.indent -= 1;
              writeln!(self.writer, ",")?;
            } else {
              self.follow_up_tasks.push(FollowUpTasks::SetAdd {
                target: *reference,
                value: value.clone(),
              });
            }
          }
          self.display_indent(0)?;
          write!(self.writer, "])")?;
        }
      }
      HeapValue::ArrayBuffer(ab) => {
        let info = self.deps.get(reference).unwrap();
        if let Some((
          backing_view_reference,
          _backing_view_info,
          _backing_view,
        )) = self.array_buffer_view_to_render_array_buffer_in(ab, info)
        {
          assert!(ab.max_byte_length.is_none());
          let ident = self.idents.get(&backing_view_reference).unwrap();
          write!(self.writer, "{}.buffer", ident)?;
        } else {
          let mut kind = ArrayBufferViewKind::Uint8Array;
          for reference in &self.order {
            if let HeapValue::ArrayBufferView(view) = reference.open(self.heap)
            {
              if view.buffer == *reference
                && view.byte_offset % view.kind.byte_width() == 0
              {
                kind = view.kind;
                break;
              }
            }
          }
          if let Some(max_byte_length) = ab.max_byte_length {
            write!(
              self.writer,
              "new ArrayBuffer({}, {{ maxByteLength: {} }})",
              ab.byte_length(),
              max_byte_length
            )?;
            if ab.data.iter().any(|b| *b != 0) {
              // if any values in the array are non 0, we need to initialize
              // them
              self.follow_up_tasks.push(FollowUpTasks::ArrayBufferSet {
                target: *reference,
                kind,
              });
            }
          } else if ab.byte_length() == 0 {
            write!(self.writer, "new ArrayBuffer()")?;
          } else if ab.data.iter().all(|b| *b == 0) {
            write!(self.writer, "new ArrayBuffer({})", ab.byte_length())?;
          } else {
            write!(self.writer, "new {}(", kind)?;
            self.display_array_buffer_data_array(kind, ab)?;
            write!(self.writer, ").buffer")?;
          }
        }
      }
      HeapValue::ArrayBufferView(view) => {
        let buffer_info = self.deps.get(&view.buffer).unwrap();
        let HeapValue::ArrayBuffer(buffer) = view.buffer.open(self.heap) else {
          unreachable!()
        };
        if let Some((
          backing_view_reference,
          _backing_view_info,
          backing_view,
        )) =
          self.array_buffer_view_to_render_array_buffer_in(buffer, buffer_info)
        {
          assert!(!view.is_backed_by_rab);
          if backing_view_reference == *reference {
            let kind = backing_view.kind;
            write!(self.writer, "new {}(", kind)?;
            if buffer.data.is_empty() {
              // no arguments
            } else if buffer.data.iter().all(|b| *b == 0) {
              let length = buffer.data.len() / kind.byte_width() as usize;
              write!(self.writer, "{}", length)?;
            } else {
              self.display_array_buffer_data_array(kind, buffer)?;
            }
            write!(self.writer, ")")?;
          } else {
            // todo: in some cases we can do the even nicer backing_view.subarray()
            let ident = self.idents.get(&backing_view_reference).unwrap();
            write!(self.writer, "new {}({}.buffer", view.kind, ident)?;
            let length =
              (view.length * view.kind.byte_width()) + view.byte_offset;
            if view.byte_offset != 0 || length != buffer.byte_length() {
              write!(self.writer, ", {}", view.byte_offset)?;
              if length != buffer.byte_length() {
                write!(self.writer, ", {}", view.length)?;
              }
            }
            write!(self.writer, ")")?;
          }
        } else {
          let ident = self.idents.get(reference).unwrap();
          write!(self.writer, "new {}({}.buffer", view.kind, ident)?;
          let length =
            (view.length * view.kind.byte_width()) + view.byte_offset;
          let needs_explicit_length = (view.is_backed_by_rab
            && !view.is_length_tracking)
            || length != buffer.byte_length();
          if view.byte_offset != 0 || needs_explicit_length {
            write!(self.writer, ", {}", view.byte_offset)?;
            if needs_explicit_length {
              write!(self.writer, ", {}", view.length)?;
            }
          }
          write!(self.writer, ")")?;
        }
      }
      HeapValue::Error(err) => {
        write!(self.writer, "new {}(", err.name)?;
        if let Some(message) = &err.message {
          self.display_string(message)?;
        }
        if let Some(cause) = &err.cause {
          if self.is_ready_to_render(cause) {
            if err.message.is_some() {
              writeln!(self.writer, ", {{")?;
            } else {
              writeln!(self.writer, "undefined, {{")?;
            }
            self.display_indent(1)?;
            write!(self.writer, "cause: ")?;
            self.indent += 1;
            self.display_value(cause)?;
            self.indent -= 1;
            writeln!(self.writer, ",")?;
            self.display_indent(0)?;
            write!(self.writer, "}}")?;
          } else {
            self
              .follow_up_tasks
              .push(FollowUpTasks::PropertyAssignment {
                target: *reference,
                key: PropertyKey::String(StringValue::new("cause".to_owned())),
                value: cause.clone(),
                requires_ordering: false,
              });
          }
        }
        write!(self.writer, ")")?;

        let stack_value = match &err.stack {
          Some(stack) => Value::String(stack.clone()),
          None => Value::Undefined,
        };
        self
          .follow_up_tasks
          .push(FollowUpTasks::PropertyAssignment {
            target: *reference,
            key: PropertyKey::String(StringValue::new("stack".to_owned())),
            value: stack_value,
            requires_ordering: false,
          });
      }
    }
    Ok(())
  }

  fn display_indent(&mut self, extra_indent: usize) -> std::fmt::Result {
    for _ in 0..self.indent + extra_indent {
      write!(self.writer, "  ")?;
    }
    Ok(())
  }

  fn display_property_key(
    &mut self,
    key: &crate::value::PropertyKey,
  ) -> std::fmt::Result {
    match key {
      crate::value::PropertyKey::String(string) => self.display_string(string),
      crate::value::PropertyKey::I32(num) => write!(self.writer, "[{}]", num),
      crate::value::PropertyKey::U32(num) => write!(self.writer, "[{}]", num),
      crate::value::PropertyKey::Double(num) => {
        write!(self.writer, "[")?;
        self.display_number(*num)?;
        write!(self.writer, "]")
      }
    }
  }

  /// Return the HeapReference to an array buffer view that is dependant on this
  /// object if any exist. This is used to determine whether the array buffer
  /// can be rendered as part of an array buffer view, or whether it needs to be
  /// rendered separately.
  ///
  /// This will always return None if the array buffer is resizable. This is
  /// because the array buffer view will need to be rendered separately in order
  /// to resize the array buffer.
  ///
  /// This method is only relevant for array buffers.
  fn array_buffer_view_to_render_array_buffer_in<'d>(
    &'d self,
    array_buffer: &ArrayBuffer,
    info: &HeapObjectInfo,
  ) -> Option<(HeapReference, &'d HeapObjectInfo, &'d ArrayBufferView)> {
    if array_buffer.max_byte_length.is_some() {
      return None;
    }
    for reference in &self.order {
      if info.dependants.contains(reference) {
        let info = self.deps.get(reference).unwrap();
        if let HeapValue::ArrayBufferView(view) = reference.open(self.heap) {
          assert!(!view.is_backed_by_rab);
          assert!(!view.is_length_tracking);
          if view.byte_offset == 0
            && (view.length * view.kind.byte_width())
              == array_buffer.byte_length()
          {
            return Some((*reference, info, view));
          }
        }
      }
    }
    None
  }

  fn display_array_buffer_data_array(
    &mut self,
    kind: ArrayBufferViewKind,
    buffer: &ArrayBuffer,
  ) -> std::fmt::Result {
    writeln!(self.writer, "[")?;

    macro_rules! display_hex_signed {
      ($name:ident, $type:ty, $suffix:literal) => {
        #[inline(always)]
        fn $name(
          this: &mut Displayer<'_, impl Write>,
          item: $type,
        ) -> std::fmt::Result {
          if item < 0 {
            write!(this.writer, concat!("-{:#04x}", $suffix), item.abs())?;
          } else {
            write!(this.writer, concat!("{:#04x}", $suffix), item)?;
          }
          Ok(())
        }
      };
    }

    #[inline(always)]
    fn display_hex_u<T: LowerHex>(
      this: &mut Displayer<'_, impl Write>,
      item: T,
    ) -> std::fmt::Result {
      write!(this.writer, "{:#04x}", item)
    }

    #[inline(always)]
    fn display_hex_u64(
      this: &mut Displayer<'_, impl Write>,
      item: u64,
    ) -> std::fmt::Result {
      write!(this.writer, "{:#04x}n", item)
    }

    display_hex_signed!(display_hex_i8, i8, "");
    display_hex_signed!(display_hex_i16, i16, "");
    display_hex_signed!(display_hex_i32, i32, "");
    display_hex_signed!(display_hex_i64, i64, "n");

    #[inline(always)]
    fn display_f32(
      this: &mut Displayer<'_, impl Write>,
      item: f32,
    ) -> std::fmt::Result {
      this.display_number_f32(item)
    }

    #[inline(always)]
    fn display_f64(
      this: &mut Displayer<'_, impl Write>,
      item: f64,
    ) -> std::fmt::Result {
      this.display_number(item)
    }

    macro_rules! write_array {
      ($type: ty, $method: ident, $display: ident) => {{
        let slice = buffer.$method();
        for (i, item) in slice.iter().enumerate() {
          if i % 8 == 0 {
            self.display_indent(1)?;
          } else {
            self.writer.write_char(' ')?;
          }
          $display(self, *item)?;
          self.writer.write_char(',')?;
          if i % 8 == 7 {
            self.writer.write_char('\n')?;
          }
        }
        if slice.len() % 8 != 0 {
          self.writer.write_char('\n')?;
        }
      }};
    }
    match kind {
      ArrayBufferViewKind::Int8Array => {
        write_array!(i8, as_i8_slice, display_hex_i8)
      }
      ArrayBufferViewKind::Uint8Array
      | ArrayBufferViewKind::Uint8ClampedArray
      | ArrayBufferViewKind::DataView => {
        write_array!(u8, as_u8_slice, display_hex_u)
      }
      ArrayBufferViewKind::Int16Array => {
        write_array!(i16, as_i16_slice, display_hex_i16)
      }
      ArrayBufferViewKind::Uint16Array => {
        write_array!(u16, as_u16_slice, display_hex_u)
      }
      ArrayBufferViewKind::Int32Array => {
        write_array!(i32, as_i32_slice, display_hex_i32)
      }
      ArrayBufferViewKind::Uint32Array => {
        write_array!(u32, as_u32_slice, display_hex_u)
      }
      ArrayBufferViewKind::Float32Array => {
        write_array!(f32, as_f32_slice, display_f32)
      }
      ArrayBufferViewKind::Float64Array => {
        write_array!(f64, as_f64_slice, display_f64)
      }
      ArrayBufferViewKind::BigInt64Array => {
        write_array!(i64, as_i64_slice, display_hex_i64)
      }
      ArrayBufferViewKind::BigUint64Array => {
        write_array!(u64, as_u64_slice, display_hex_u64)
      }
    }

    self.display_indent(0)?;
    write!(self.writer, "]")?;

    Ok(())
  }
}

pub fn display(heap: &Heap, value: &Value, opts: DisplayOptions) -> String {
  let mut result = String::new();
  Displayer::display(heap, value, opts, &mut result).unwrap();
  result
}

pub enum DisplayFormat {
  /// Display the value as a string that can be passed to a JavaScript REPL. In
  /// this mode, the final statement of the string is not terminated with a
  /// semicolon and may be wrapped in parentheses to make it an expression
  /// statement.
  ///
  /// This is usually the clearest way to display a value to a user.
  Repl,
  /// Display the value as a string that can be used in an expression position,
  /// such as the RHS of a function. If intermediate variables are needed, the
  /// entire string is wrapped in an IIFE.
  Expression,
  /// Display the value as a string that can be passed to eval(). One should use
  /// indirect eval (for example `(0, eval)()`) to avoid polluting the current
  /// scope.
  Eval,
}

pub struct DisplayOptions {
  pub format: DisplayFormat,
}
