use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Write;

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
}

struct Displayer<'h, W: Write> {
  heap: &'h Heap,
  writer: W,
  indent: usize,
  /// A map that keeps track of which heap objects have been seen, and what heap
  /// objects they reference and are referenced by. This is used to track cycles
  /// and determine whether an object can be inlined or not.
  deps: HashMap<HeapReference, HeapObjectInfo>,
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
  fn inlineable(&self) -> bool {
    self.dependants_count < 2 && !self.requires_binding
  }
}

impl<'h, W: Write> Displayer<'h, W> {
  fn display(heap: &'h Heap, value: &Value, writer: W) -> std::fmt::Result {
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
          for value in &arr.elements {
            if let Some(Value::HeapReference(referred)) = value {
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
        HeapValue::ArrayBuffer(_) => {}
        HeapValue::ArrayBufferView(view) => {
          visit_and_record!(heap, deps, referrer, view.buffer);
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

    println!("{:#?}", deps);

    let mut this = Self {
      heap,
      writer,
      indent: 0,
      deps: deps.objects,
      idents: HashMap::new(),
      follow_up_tasks: Vec::new(),
    };
    let Value::HeapReference(reference) = value else {
      return this.display_value(value);
    };
    assert_eq!(deps.order.last(), Some(reference));

    for (i, reference) in deps.order.iter().enumerate() {
      let info = this.deps.get(reference).unwrap();
      if !info.inlineable() {
        // we need to assign this object to a variable because it is referenced
        // by multiple objects
        let ident = format!("v{}", i);
        write!(this.writer, "const {} = ", ident)?;
        this.display_heap_value(reference.open(this.heap), reference)?;
        writeln!(this.writer, ";")?;
        this.idents.insert(*reference, ident.clone());

        // Run the follow up tasks for this object.
        let follow_up_tasks_to_run = std::mem::take(&mut this.follow_up_tasks);
        for task in follow_up_tasks_to_run {
          match &task {
            FollowUpTasks::PropertyAssignment { target, key, value } => {
              let Some(ident) = this.idents.get(&target) else {
                this.follow_up_tasks.push(task);
                continue;
              };
              if !this.is_ready_to_render(value) {
                this.follow_up_tasks.push(task);
                continue;
              }
              write!(this.writer, "{}[", ident)?;
              this.display_property_key(&key)?;
              write!(this.writer, "] = ")?;
              this.display_value(&value)?;
              writeln!(this.writer, ";")?;
            }
            FollowUpTasks::MapSet { target, key, value } => {
              let Some(ident) = this.idents.get(&target) else {
                this.follow_up_tasks.push(task);
                continue;
              };
              if !this.is_ready_to_render(&key)
                || !this.is_ready_to_render(&value)
              {
                this.follow_up_tasks.push(task);
                continue;
              }
              write!(this.writer, "{}.set(", ident)?;
              this.display_value(&key)?;
              write!(this.writer, ", ")?;
              this.display_value(&value)?;
              writeln!(this.writer, ");")?;
            }
            FollowUpTasks::SetAdd { target, value } => {
              let Some(ident) = this.idents.get(&target) else {
                this.follow_up_tasks.push(task);
                continue;
              };
              if !this.is_ready_to_render(&value) {
                this.follow_up_tasks.push(task);
                continue;
              }
              write!(this.writer, "{}.add(", ident)?;
              this.display_value(&value)?;
              writeln!(this.writer, ");")?;
            }
          }
        }
      }
    }

    assert!(this.follow_up_tasks.is_empty());

    this.display_value(value)
  }

  fn is_ready_to_render(&self, value: &Value) -> bool {
    match value {
      Value::HeapReference(reference) => {
        self.deps.get(reference).unwrap().inlineable()
          || self.idents.get(reference).is_some()
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
          // todo: circular references
          let heap_value = reference.open(&self.heap);
          self.display_heap_value(heap_value, reference)
        }
      }
    }
  }

  fn display_string(&mut self, string: &StringValue) -> std::fmt::Result {
    match string {
      StringValue::Utf8(string) => self.display_string_literal(string.chars()),
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
        write!(self.writer, "BigInt({})", val)?;
      }
      HeapValue::StringObject(str) => {
        write!(self.writer, "new String(")?;
        self.display_string(str)?;
        write!(self.writer, ")")?;
      }
      HeapValue::RegExp(regexp) => {
        write!(self.writer, "new RegExp(")?;
        self.display_string(&regexp.pattern)?;
        write!(self.writer, ", ")?;
        write!(self.writer, ")")?;
        todo!("regexp flags");
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
          if self.is_ready_to_render(value) {
            self.display_indent(1)?;
            self.display_property_key(key)?;
            write!(self.writer, ": ")?;
            self.indent += 1;
            self.display_value(value)?;
            self.indent -= 1;
            writeln!(self.writer, ",")?;
          } else {
            self
              .follow_up_tasks
              .push(FollowUpTasks::PropertyAssignment {
                target: *reference,
                key: key.clone(),
                value: value.clone(),
              });
          }
        }
        self.display_indent(0)?;
        write!(self.writer, "}}")?;
      }
      HeapValue::SparseArray(arr) => {
        write!(self.writer, "new Array({})", arr.length)?;
        if arr.properties.len() > 0 {
          for (key, value) in &arr.properties {
            self
              .follow_up_tasks
              .push(FollowUpTasks::PropertyAssignment {
                target: *reference,
                key: key.clone(),
                value: value.clone(),
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
              self
                .follow_up_tasks
                .push(FollowUpTasks::PropertyAssignment {
                  target: *reference,
                  key: PropertyKey::I32(i as i32),
                  value: value.clone(),
                });
              write!(self.writer, "null")?;
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
            });
        }
      }
      HeapValue::Map(map) => {
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
      HeapValue::Set(set) => {
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
      HeapValue::ArrayBuffer(_) => todo!(),
      HeapValue::ArrayBufferView(_) => todo!(),
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
    }
  }
}

pub fn display(heap: &Heap, value: &Value) -> String {
  let mut result = String::new();
  Displayer::display(&heap, &value, &mut result).unwrap();
  result
}
