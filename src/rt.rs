use std::{cmp::{Ordering, Reverse}, collections::BTreeMap, fs::File, io::{self, BufReader}, iter, mem::transmute, path::Path, str::from_utf8_unchecked};

use crate::{class::{AttributeInfo, ClassFile, ConstIndex, Constant, FieldAccess, MethodAccess}, code::opcode::Opcode, descriptor::{DescriptorError, FieldDescriptor, MethodDescriptor}};

mod bytes;
mod builtin_methods;

pub use bytes::*;
use collect_result::CollectResult;
use num_enum::FromPrimitive;
pub type Result<T, E=RtError> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct RuntimeCtx<'a> {
    runtime: &'a mut Runtime,
    stack: Vec<Value>,
    return_stack: Vec<(u32, u32, usize, u16)>,
    frame_pointer: u32,
    max_locals: u16,
    cur_class: u32,

    heap: Vec<u32>,

    pc: usize,
}
impl RuntimeCtx<'_> {
    pub fn top(&self) -> Value {
        *self.stack.last().unwrap()
    }
    pub fn top2(&self) -> (Value, Value) {
        let l = self.stack.len();
        (self.stack[l-2], self.stack[l-1])
    }
    pub fn pop(&mut self) -> Value {
        assert!(self.stack.len() > (self.frame_pointer + self.max_locals as u32) as usize);
        let val = self.stack.pop().unwrap();
        eprintln!("pop {val:?}");
        val
    }
    pub fn pop2(&mut self) -> (Value, Value) {
        assert!(self.stack.len() > 1 + (self.frame_pointer + self.max_locals as u32) as usize);
        let v2 = self.stack.pop().unwrap();
        let val = (self.stack.pop().unwrap(), v2);
        eprintln!("pop {val:?}");
        val
    }
    pub fn push(&mut self, value: impl Into<Value>) {
        let val = value.into();
        eprintln!("push {val:?}");
        self.stack.push(val);
    }
    pub fn push2(&mut self, value: impl Into<(Value, Value)>) {
        let (v1, v2) = value.into();
        eprintln!("push {:?}", (v1, v2));
        self.stack.push(v1);
        self.stack.push(v2);
    }
    pub fn get_local(&self, i: u16) -> Value {
        assert!(i < self.max_locals);
        let val = self.stack[self.frame_pointer as usize + i as usize];
        eprintln!("get {i} {val:?}");
        val
    }
    pub fn get_local2(&self, i: u16) -> (Value, Value) {
        assert!(i < self.max_locals);
        let i = self.frame_pointer as usize + i as usize;
        let val = (self.stack[i], self.stack[i + 1]);
        eprintln!("get {i} {val:?}");
        val
    }
    pub fn set_local(&mut self, i: u16, value: impl Into<Value>) {
        assert!(i < self.max_locals);
        let val = value.into();
        eprintln!("set {i} {val:?}");
        self.stack[self.frame_pointer as usize + i as usize] = val;
    }
    pub fn set_local2(&mut self, i: u16, value: impl Into<(Value, Value)>) {
        assert!(i < self.max_locals);
        let i = self.frame_pointer as usize + i as usize;
        let (v1, v2) = value.into();
        eprintln!("set {i} {:?}", (v1, v2));
        self.stack[i] = v1;
        self.stack[i + 1] = v2;
    }
    pub fn get_class_name(&self, _reference: Value) -> &str {
        todo!()
    }
    pub fn get_class_object(&self, _reference: Value) -> Value {
        todo!()
    }
    pub fn new_string_obj(&self, _s: impl Into<Box<str>>) -> Value {
        todo!()
    }
    pub fn call_named(&mut self, classpath: &str, method_name: &str, method_type: MethodDescriptor) -> Result<()> {
        let id = self.runtime.load_class(classpath)?;
        let method_id = self.runtime.classes[id as usize].member_table()[(method_name, &method_type.into())];
        self.invoke(id, method_id);
        Ok(())
    }
    pub fn invoke(&mut self, class: u32, method_id: u16) {
        match self.runtime.classes[class as usize].method(method_id) {
            Err(f) => f(self),
            Ok(bm) => {
                self.stack.reserve((bm.max_locals - bm.arg_num) as usize + bm.max_stack as usize);
                self.do_call(class, bm.arg_num, bm.max_locals, bm.code_location);
            }
        }
    }
    pub fn do_call(&mut self, class: u32, arg_num: u16, max_locals: u16, location: usize) {
        self.return_stack.push((self.cur_class, self.frame_pointer, self.pc, self.max_locals));
        self.frame_pointer = (self.stack.len() - arg_num as usize) as u32;
        self.pc = location;
        self.cur_class = class;
        self.max_locals = max_locals;
        for _ in arg_num..max_locals {
            self.stack.push(Value(0));
        }
    }
    pub fn do_return(&mut self, ret_cat: ReturnCategory) {
        let (class, fp, pc, max_locals) = self.return_stack.pop().unwrap();
        eprintln!("return, throwing away: {:?}", &self.stack[self.frame_pointer as usize..]);
        self.pc = pc;
        self.cur_class = class;
        match ret_cat {
            ReturnCategory::Void => {
                self.stack.truncate(self.frame_pointer as usize);
                self.frame_pointer = fp;
            }
            ReturnCategory::Cat1 => {
                let return_value = self.pop();
                self.stack.truncate(self.frame_pointer as usize);
                self.frame_pointer = fp;
                self.push(return_value);
            }
            ReturnCategory::Cat2 => {
                let return_value = self.pop2();
                self.stack.truncate(self.frame_pointer as usize);
                self.frame_pointer = fp;
                self.push2(return_value);
            }
        }
        self.max_locals = max_locals;
    }
    #[track_caller]
    fn read_constant(&self, n: ConstIndex) -> RuntimeConstant {
        match &self.runtime.classes[self.cur_class as usize].runtime_info {
            RuntimeInfo::Bytecode { constant_pool, .. } => {
                constant_pool[n as usize - 1]
            }
            _ => unimplemented!(),
        }
    }
    fn new(&mut self, class_id: u32) -> Value {
        let size = self.runtime.get_class(class_id).get_aligned_data_size();
        let i = self.heap.as_bytes_32aligned().len();
        for _ in 0..size/4 {
            self.heap.push(0);
        }
        Value::new_ref_heap(i as u32)
    }
    fn read_u8_ref(&self, ptr: Value) -> Option<u8> {
        Some(match ptr.into_ref() {
            Reference::Invalid => return None,
            Reference::Heap(offset) => self.heap.as_bytes_32aligned()[offset as usize],
            Reference::Static(offset) => self.runtime.statics.as_bytes_32aligned()[offset as usize],
        })
    }
    fn read_u16_ref(&self, ptr: Value) -> Option<u16> {
        debug_assert_eq!(ptr.into_u32() & 1, 0);
        let ptr = match ptr.into_ref() {
            Reference::Invalid => return None,
            Reference::Heap(offset) => &self.heap.as_bytes_32aligned()[offset as usize],
            Reference::Static(offset) => &self.runtime.statics.as_bytes_32aligned()[offset as usize],
        };
        Some(unsafe {*(ptr as *const u8 as *const u16)})
    }
    fn read_u32_ref(&self, ptr: Value) -> Option<u32> {
        Some(match ptr.into_ref() {
            Reference::Invalid => return None,
            Reference::Heap(offset) => {
                debug_assert_eq!(offset & 3, 0);
                self.heap[offset as usize / 4]
            }
            Reference::Static(offset) => {
                debug_assert_eq!(offset & 3, 0);
                self.runtime.statics[offset as usize / 4]
            }
        })
    }
    fn write_u8_ref(&mut self, ptr: Value, val: u8) {
        match ptr.into_ref() {
            Reference::Invalid => unimplemented!(),
            Reference::Heap(offset) => self.heap.as_bytes_32aligned_mut()[offset as usize] = val,
            Reference::Static(offset) => self.runtime.statics.as_bytes_32aligned_mut()[offset as usize] = val,
        }
    }
    fn write_u16_ref(&mut self, ptr: Value, val: u16) {
        debug_assert_eq!(ptr.into_u32() & 1, 0);
        let ptr = match ptr.into_ref() {
            Reference::Invalid => unimplemented!(),
            Reference::Heap(offset) => &mut self.heap.as_bytes_32aligned_mut()[offset as usize],
            Reference::Static(offset) => &mut self.runtime.statics.as_bytes_32aligned_mut()[offset as usize],
        };
        unsafe {*(ptr as *mut u8 as *mut u16) = val};
    }
    fn write_u32_ref(&mut self, ptr: Value, val: u32) {
        match ptr.into_ref() {
            Reference::Invalid => unimplemented!(),
            Reference::Heap(offset) => {
                debug_assert_eq!(offset & 3, 0);
                self.heap[offset as usize / 4] = val;
            }
            Reference::Static(offset) => {
                debug_assert_eq!(offset & 3, 0);
                self.runtime.statics[offset as usize / 4] = val;
            }
        }
    }
    fn decode_opcode(&mut self) -> Opcode {
        let code = self.runtime.code.as_bytes_32aligned();
        let opcode = Opcode::from_primitive(code[self.pc]);
        self.pc += 1;
        opcode
    }
    fn decode_u8(&mut self) -> u8 {
        let code = self.runtime.code.as_bytes_32aligned();
        let byte = code[self.pc];
        self.pc += 1;
        byte
    }
    fn decode_u16(&mut self) -> u16 {
        let code = self.runtime.code.as_bytes_32aligned();
        let short = u16::from_be_bytes([code[self.pc], code[self.pc+1]]);
        self.pc += 2;
        short
    }
    fn do_goto(&mut self, instruction_length: u8, offset: i16) {
        self.pc = (self.pc - instruction_length as usize).wrapping_add(offset as isize as usize);
    }
    fn run_inner(&mut self) -> Result<()> {
        while !self.return_stack.is_empty() {
            let opcode = self.decode_opcode();
            eprintln!("  ins {}", opcode.mnemonic());
            match opcode {
                Opcode::Nop => (),
                Opcode::AconstNull |
                Opcode::Iconst0 => self.push(0),
                Opcode::IconstM1 => self.push(-1),
                Opcode::Iconst1 => self.push(1),
                Opcode::Iconst2 => self.push(2),
                Opcode::Iconst3 => self.push(3),
                Opcode::Iconst4 => self.push(4),
                Opcode::Iconst5 => self.push(5),
                Opcode::Lconst0 => self.push2(i64_into_values(0)),
                Opcode::Lconst1 => self.push2(i64_into_values(1)),
                Opcode::Fconst0 => self.push(0.),
                Opcode::Fconst1 => self.push(1.),
                Opcode::Fconst2 => self.push(2.),
                Opcode::Dconst0 => self.push2(f64_into_values(0.)),
                Opcode::Dconst1 => self.push2(f64_into_values(1.)),
                Opcode::Bipush => {
                    let imm = self.decode_u8() as i8;
                    self.push(imm);
                }
                Opcode::Sipush => {
                    let imm = self.decode_u16() as i16;
                    self.push(imm);
                }
                Opcode::Ldc => {
                    let cpn = self.decode_u8();
                    // TODO: DANGER the constant might not actually be a string!!!
                    let cpn = self.read_constant(cpn as u16).string();
                    let s_offset = self.read_constant(cpn).utf8();
                    self.push(Value::new_ref_static(s_offset));
                }
                Opcode::LdcW => {
                    let cpn = self.decode_u16();
                    // TODO: DANGER the constant might not actually be a string!!!
                    let cpn = self.read_constant(cpn as u16).string();
                    let s_offset = self.read_constant(cpn).utf8();
                    self.push(Value::new_ref_static(s_offset));
                }
                Opcode::Ldc2W => todo!(),
                Opcode::Iload |
                Opcode::Fload |
                Opcode::Aload => {
                    let index = self.decode_u8();
                    let val = self.get_local(index as u16);
                    self.push(val);
                },
                Opcode::Lload |
                Opcode::Dload => {
                    let index = self.decode_u8();
                    let val = self.get_local2(index as u16);
                    self.push2(val);
                }
                Opcode::Iload0 |
                Opcode::Fload0 |
                Opcode::Aload0 => {
                    let val = self.get_local(0);
                    self.push(val);
                }
                Opcode::Lload0 |
                Opcode::Dload0 => {
                    let val = self.get_local2(0);
                    self.push2(val);
                }
                Opcode::Iload1 |
                Opcode::Fload1 |
                Opcode::Aload1 => {
                    let val = self.get_local(1);
                    self.push(val);
                }
                Opcode::Lload1 |
                Opcode::Dload1 => {
                    let val = self.get_local2(1);
                    self.push2(val);
                }
                Opcode::Iload2 |
                Opcode::Fload2 |
                Opcode::Aload2  => {
                    let val = self.get_local(2);
                    self.push(val);
                }
                Opcode::Lload2 |
                Opcode::Dload2  => {
                    let val = self.get_local2(2);
                    self.push2(val);
                }
                Opcode::Iload3 |
                Opcode::Fload3 |
                Opcode::Aload3  => {
                    let val = self.get_local(3);
                    self.push(val);
                }
                Opcode::Lload3 |
                Opcode::Dload3  => {
                    let val = self.get_local2(3);
                    self.push2(val);
                }
                Opcode::Faload |
                Opcode::Iaload |
                Opcode::Aaload => {
                    let index = self.pop().into_u32();
                    let arr_ref = self.pop();
                    let length = self.read_u32_ref(arr_ref).unwrap();
                    if index >= length {
                        unimplemented!("out of bounds");
                    }
                    let val = self.read_u32_ref(arr_ref.offset(4*index)).unwrap();
                    self.push(Value(val));
                }
                Opcode::Laload |
                Opcode::Daload => {
                    let index = self.pop().into_u32();
                    let arr_ref = self.pop();
                    let length = self.read_u32_ref(arr_ref).unwrap();
                    if index >= length {
                        unimplemented!("out of bounds");
                    }
                    let val1 = self.read_u32_ref(arr_ref.offset(4*index)).unwrap();
                    let val2 = self.read_u32_ref(arr_ref.offset(4*index+4)).unwrap();
                    self.push2((Value(val1), Value(val2)));
                }
                Opcode::Baload => {
                    let index = self.pop().into_u32();
                    let arr_ref = self.pop();
                    let length = self.read_u32_ref(arr_ref).unwrap();
                    if index >= length {
                        unimplemented!("out of bounds");
                    }
                    let val = self.read_u8_ref(arr_ref.offset(index)).unwrap();
                    self.push(val as i8);
                }
                Opcode::Caload |
                Opcode::Saload => {
                    let index = self.pop().into_u32();
                    let arr_ref = self.pop();
                    let length = self.read_u32_ref(arr_ref).unwrap();
                    if index >= length {
                        unimplemented!("out of bounds");
                    }
                    let val = self.read_u16_ref(arr_ref.offset(index)).unwrap();
                    self.push(val as i16);
                }
                Opcode::Istore |
                Opcode::Fstore |
                Opcode::Astore => {
                    let index = self.decode_u8();
                    let value = self.pop();
                    self.set_local(index as u16, value);
                }
                Opcode::Lstore |
                Opcode::Dstore => {
                    let index = self.decode_u8();
                    let values = self.pop2();
                    self.set_local2(index as u16, values);
                }
                Opcode::Istore0 |
                Opcode::Fstore0 |
                Opcode::Astore0 => {
                    let value = self.pop();
                    self.set_local(0, value);
                }
                Opcode::Istore1 |
                Opcode::Fstore1 |
                Opcode::Astore1 => {
                    let value = self.pop();
                    self.set_local(1, value);
                }
                Opcode::Istore2 |
                Opcode::Fstore2 |
                Opcode::Astore2 => {
                    let value = self.pop();
                    self.set_local(2, value);
                }
                Opcode::Istore3 |
                Opcode::Fstore3 |
                Opcode::Astore3 => {
                    let value = self.pop();
                    self.set_local(3, value);
                }
                Opcode::Lstore0 |
                Opcode::Dstore0 => {
                    let values = self.pop2();
                    self.set_local2(0, values);
                }
                Opcode::Lstore1 |
                Opcode::Dstore1 => {
                    let values = self.pop2();
                    self.set_local2(1, values);
                }
                Opcode::Lstore2 |
                Opcode::Dstore2 => {
                    let values = self.pop2();
                    self.set_local2(2, values);
                }
                Opcode::Lstore3 |
                Opcode::Dstore3 => {
                    let values = self.pop2();
                    self.set_local2(3, values);
                }
                Opcode::Iastore => todo!(),
                Opcode::Lastore => todo!(),
                Opcode::Fastore => todo!(),
                Opcode::Dastore => todo!(),
                Opcode::Aastore => todo!(),
                Opcode::Bastore => todo!(),
                Opcode::Castore => todo!(),
                Opcode::Sastore => todo!(),
                Opcode::Pop => {
                    self.pop();
                }
                Opcode::Pop2 => {
                    self.pop2();
                }
                Opcode::Dup => self.push(self.top()),
                Opcode::DupX1 => todo!(),
                Opcode::DupX2 => todo!(),
                Opcode::Dup2 => todo!(),
                Opcode::Dup2X1 => todo!(),
                Opcode::Dup2X2 => todo!(),
                Opcode::Swap => {
                    let v1 = self.pop();
                    let v2 = self.pop();
                    self.push(v1);
                    self.push(v2);
                }
                Opcode::Iadd => {
                    let v2 = self.pop().into_u32() as i32;
                    let v1 = self.pop().into_u32() as i32;
                    self.push(v1+v2);
                }
                Opcode::Ladd => todo!(),
                Opcode::Fadd => todo!(),
                Opcode::Dadd => todo!(),
                Opcode::Isub => {
                    let v2 = self.pop().into_u32() as i32;
                    let v1 = self.pop().into_u32() as i32;
                    self.push(v1-v2);
                }
                Opcode::Lsub => todo!(),
                Opcode::Fsub => todo!(),
                Opcode::Dsub => todo!(),
                Opcode::Imul => {
                    let v2 = self.pop().into_u32() as i32;
                    let v1 = self.pop().into_u32() as i32;
                    self.push(v1*v2);
                }
                Opcode::Lmul => todo!(),
                Opcode::Fmul => todo!(),
                Opcode::Dmul => todo!(),
                Opcode::Idiv => {
                    let v2 = self.pop().into_u32() as i32;
                    let v1 = self.pop().into_u32() as i32;
                    self.push(v1/v2);
                }
                Opcode::Ldiv => todo!(),
                Opcode::Fdiv => todo!(),
                Opcode::Ddiv => todo!(),
                Opcode::Irem => todo!(),
                Opcode::Lrem => todo!(),
                Opcode::Frem => todo!(),
                Opcode::Drem => todo!(),
                Opcode::Ineg => todo!(),
                Opcode::Lneg => todo!(),
                Opcode::Fneg => todo!(),
                Opcode::Dneg => todo!(),
                Opcode::Ishl => todo!(),
                Opcode::Lshl => todo!(),
                Opcode::Ishr => todo!(),
                Opcode::Lshr => todo!(),
                Opcode::Iushr => todo!(),
                Opcode::Lushr => todo!(),
                Opcode::Iand => todo!(),
                Opcode::Land => todo!(),
                Opcode::Ior => todo!(),
                Opcode::Lor => todo!(),
                Opcode::Ixor => todo!(),
                Opcode::Lxor => todo!(),
                Opcode::Iinc => todo!(),
                Opcode::I2l => todo!(),
                Opcode::I2f => todo!(),
                Opcode::I2d => todo!(),
                Opcode::L2i => todo!(),
                Opcode::L2f => todo!(),
                Opcode::L2d => todo!(),
                Opcode::F2i => todo!(),
                Opcode::F2l => todo!(),
                Opcode::F2d => todo!(),
                Opcode::D2i => todo!(),
                Opcode::D2l => todo!(),
                Opcode::D2f => todo!(),
                Opcode::I2b => todo!(),
                Opcode::I2c => todo!(),
                Opcode::I2s => todo!(),
                Opcode::Lcmp => {
                    let value2 = values_into_u64(self.pop2()) as i64;
                    let value1 = values_into_u64(self.pop2()) as i64;
                    self.push(value1.cmp(&value2) as i8);
                }
                Opcode::Fcmpl => {
                    let value2 = self.pop().into_f32();
                    let value1 = self.pop().into_f32();
                    self.push(value1.partial_cmp(&value2).unwrap_or(Ordering::Less) as i8);
                }
                Opcode::Fcmpg => {
                    let value2 = self.pop().into_f32();
                    let value1 = self.pop().into_f32();
                    self.push(value1.partial_cmp(&value2).unwrap_or(Ordering::Greater) as i8);
                }
                Opcode::Dcmpl => {
                    let value2 = values_into_f64(self.pop2());
                    let value1 = values_into_f64(self.pop2());
                    self.push(value1.partial_cmp(&value2).unwrap_or(Ordering::Less) as i8);
                }
                Opcode::Dcmpg => {
                    let value2 = values_into_f64(self.pop2());
                    let value1 = values_into_f64(self.pop2());
                    self.push(value1.partial_cmp(&value2).unwrap_or(Ordering::Greater) as i8);
                }
                Opcode::Ifnull |
                Opcode::Ifeq => {
                    let offset = self.decode_u16() as i16;
                    if self.pop() == Value::ZERO {
                        self.do_goto(3, offset);
                    }
                }
                Opcode::Ifnonnull |
                Opcode::Ifne => {
                    let offset = self.decode_u16() as i16;
                    if self.pop() != Value::ZERO {
                        self.do_goto(3, offset);
                    }
                }
                Opcode::Iflt => {
                    let offset = self.decode_u16() as i16;
                    if self.pop().into_i32() < 0 {
                        self.do_goto(3, offset);
                    }
                }
                Opcode::Ifge => {
                    let offset = self.decode_u16() as i16;
                    if self.pop().into_i32() >= 0 {
                        self.do_goto(3, offset);
                    }
                }
                Opcode::Ifgt => {
                    let offset = self.decode_u16() as i16;
                    if self.pop().into_i32() > 0 {
                        self.do_goto(3, offset);
                    }
                }
                Opcode::Ifle => {
                    let offset = self.decode_u16() as i16;
                    if self.pop().into_i32() <= 0 {
                        self.do_goto(3, offset);
                    }
                }
                Opcode::IfAcmpeq |
                Opcode::IfIcmpeq => {
                    let offset = self.decode_u16() as i16;
                    if self.pop() == self.pop() {
                        self.do_goto(3, offset);
                    }
                }
                Opcode::IfAcmpne |
                Opcode::IfIcmpne => {
                    let offset = self.decode_u16() as i16;
                    if self.pop() != self.pop() {
                        self.do_goto(3, offset);
                    }
                }
                Opcode::IfIcmplt => {
                    let offset = self.decode_u16() as i16;
                    if self.pop().into_i32() < self.pop().into_i32() {
                        self.do_goto(3, offset);
                    }
                }
                Opcode::IfIcmpge => {
                    let offset = self.decode_u16() as i16;
                    if self.pop().into_i32() >= self.pop().into_i32() {
                        self.do_goto(3, offset);
                    }
                }
                Opcode::IfIcmpgt => {
                    let offset = self.decode_u16() as i16;
                    if self.pop().into_i32() > self.pop().into_i32() {
                        self.do_goto(3, offset);
                    }
                }
                Opcode::IfIcmple => {
                    let offset = self.decode_u16() as i16;
                    if self.pop().into_i32() <= self.pop().into_i32() {
                        self.do_goto(3, offset);
                    }
                }
                Opcode::Goto => {
                    let offset = self.decode_u16() as i16;
                    self.do_goto(3, offset);
                }
                Opcode::Jsr => todo!(),
                Opcode::Ret => todo!(),
                Opcode::Tableswitch => todo!(),
                Opcode::Lookupswitch => todo!(),
                Opcode::Lreturn |
                Opcode::Dreturn => self.do_return(ReturnCategory::Cat2),
                Opcode::Ireturn |
                Opcode::Freturn |
                Opcode::Areturn => self.do_return(ReturnCategory::Cat1),
                Opcode::Return => self.do_return(ReturnCategory::Void),
                Opcode::Getstatic => {
                    let cpn = self.decode_u16();
                    let (class_index, name_and_type_index) = self.read_constant(cpn).fieldref();
                    let class = self.read_constant(self.read_constant(class_index).class()).utf8();
                    let (name, field_type) = self.read_constant(name_and_type_index).nameandtype();
                    let name = self.read_constant(name).utf8();
                    let field_type = self.read_constant(field_type).utf8();
                    let field_type = FieldDescriptor::from_bytes(self.runtime.read_static_string(field_type).as_bytes())?;

                    let ptr = {
                        let class = self.runtime.read_static_string(class).to_string();
                        let class = self.runtime.load_class(&class)?;
                        let class = self.runtime.get_class(class);
                        let name = self.runtime.read_static_string(name);
                        let offset = class.member_table[(name, &field_type.clone().into())];
                        &class.static_fields[offset as usize]
                    };

                    match field_type {
                        FieldDescriptor::Boolean |
                        FieldDescriptor::Byte => {
                            let value = *ptr;
                            self.push(value as i8);
                        }
                        FieldDescriptor::Char |
                        FieldDescriptor::Short => {
                            let value = unsafe { *(ptr as *const u8 as *const u16) };
                            self.push(value as i16);
                        }
                        FieldDescriptor::ClassRef(_) |
                        FieldDescriptor::ArrRef(_) |
                        FieldDescriptor::Float |
                        FieldDescriptor::Int => {
                            let value = unsafe { *(ptr as *const u8 as *const u32) };
                            self.push(value as i32);
                        }
                        FieldDescriptor::Double |
                        FieldDescriptor::Long => {
                            let ptr = ptr as *const u8 as *const u32;
                            let v1 = unsafe { *ptr };
                            let v2 = unsafe { *ptr.add(1) };
                            self.push2((Value(v1), Value(v2)));
                        }
                    }
                }
                Opcode::Getfield => {
                    let cpn = self.decode_u16();
                    let (class_index, name_and_type_index) = self.read_constant(cpn).fieldref();
                    let class = self.read_constant(self.read_constant(class_index).class()).utf8();
                    let (name, field_type) = self.read_constant(name_and_type_index).nameandtype();
                    let name = self.read_constant(name).utf8();
                    let field_type = self.read_constant(field_type).utf8();
                    let field_type = FieldDescriptor::from_bytes(self.runtime.read_static_string(field_type).as_bytes())?;

                    let offset = {
                        let class = self.runtime.read_static_string(class).to_string();
                        let class = self.runtime.load_class(&class)?;
                        let class = self.runtime.get_class(class);
                        let name = self.runtime.read_static_string(name);
                        class.member_table[(name, &field_type.clone().into())]
                    };
                    let object_ref = self.pop();

                    match field_type {
                        FieldDescriptor::Boolean |
                        FieldDescriptor::Byte => {
                            let value = self.read_u8_ref(object_ref.offset(offset as u32)).unwrap();
                            self.push(value as i8);
                        }
                        FieldDescriptor::Char |
                        FieldDescriptor::Short => {
                            let value = self.read_u16_ref(object_ref.offset(offset as u32)).unwrap();
                            self.push(value as i16);
                        }
                        FieldDescriptor::ClassRef(_) |
                        FieldDescriptor::ArrRef(_) |
                        FieldDescriptor::Float |
                        FieldDescriptor::Int => {
                            let value = self.read_u32_ref(object_ref.offset(offset as u32)).unwrap();
                            self.push(value as i32);
                        }
                        FieldDescriptor::Double |
                        FieldDescriptor::Long => {
                            let v1 = self.read_u32_ref(object_ref.offset(offset as u32)).unwrap();
                            let v2 = self.read_u32_ref(object_ref.offset(offset as u32 + 4)).unwrap();
                            self.push2((Value(v1), Value(v2)));
                        }
                    }
                }
                Opcode::Putstatic => {

                }
                Opcode::Putfield => {
                    // TODO: no unsafe in rt, all unsafe should be in its own module
                    let cpn = self.decode_u16();
                    let (class_index, name_and_type_index) = self.read_constant(cpn).fieldref();
                    let class = self.read_constant(self.read_constant(class_index).class()).utf8();
                    let (name, field_type) = self.read_constant(name_and_type_index).nameandtype();
                    let name = self.read_constant(name).utf8();
                    let field_type = self.read_constant(field_type).utf8();
                    let field_type = FieldDescriptor::from_bytes(self.runtime.read_static_string(field_type).as_bytes())?;

                    let offset = {
                        let class = self.runtime.read_static_string(class).to_string();
                        let class = self.runtime.load_class(&class)?;
                        let class = self.runtime.get_class(class);
                        let name = self.runtime.read_static_string(name);
                        class.member_table()[(name, &field_type.clone().into())]
                    };

                    match field_type {
                        FieldDescriptor::Boolean |
                        FieldDescriptor::Byte => {
                            let value = self.pop().into_u8();
                            let object_ref = self.pop();
                            self.write_u8_ref(object_ref.offset(offset as u32), value);
                        }
                        FieldDescriptor::Char |
                        FieldDescriptor::Short => {
                            let value = self.pop().into_u16();
                            let object_ref = self.pop();
                            self.write_u16_ref(object_ref.offset(offset as u32), value);
                        }
                        FieldDescriptor::ClassRef(_) |
                        FieldDescriptor::ArrRef(_) |
                        FieldDescriptor::Float |
                        FieldDescriptor::Int => {
                            let value = self.pop().into_u32();
                            let object_ref = self.pop();
                            self.write_u32_ref(object_ref.offset(offset as u32), value);
                        }
                        FieldDescriptor::Double |
                        FieldDescriptor::Long => {
                            let (v1, v2) = self.pop2();
                            let v1 = v1.into_u32();
                            let v2 = v2.into_u32();
                            let object_ref = self.pop();
                            self.write_u32_ref(object_ref.offset(offset as u32), v1);
                            self.write_u32_ref(object_ref.offset(offset as u32 + 4), v2);
                        }
                    }
                }
                Opcode::Invokevirtual |
                Opcode::Invokespecial => {
                    let cpn = self.decode_u16();
                    let (class_index, name_and_type_index) = self.read_constant(cpn).methodref();
                    let class_name = self.read_constant(self.read_constant(class_index).class()).utf8();
                    let class_name = self.runtime.read_static_string(class_name);
                    let (name_index, descriptor_index) = self.read_constant(name_and_type_index).nameandtype();
                    let name = self.read_constant(name_index).utf8();
                    let name = self.runtime.read_static_string(name);
                    let method_type = self.read_constant(descriptor_index).utf8();
                    let method_type = self.runtime.read_static_string(method_type);
                    let method_type = MethodDescriptor::from_bytes(method_type.as_bytes())?;
                    eprintln!("Invoking {class_name} {}", method_type.display_type(&name));
                    let class_name = class_name.to_string(); // TODO: bad
                    let name = name.to_string(); // TODO: bad
                    self.call_named(&class_name, &name, method_type)?;
                }
                Opcode::Invokestatic => todo!(),
                Opcode::Invokeinterface => todo!(),
                Opcode::Invokedynamic => todo!(),
                Opcode::New => {
                    let cpn = self.decode_u16();
                    let class_name = self.read_constant(self.read_constant(cpn).class()).utf8();
                    let class_name = self.runtime.read_static_string(class_name);
                    let class_name = class_name.to_string(); // TODO: bad
                    let id = self.runtime.load_class(&class_name)?;
                    let r = self.new(id);
                    self.push(r);
                }
                Opcode::Newarray => todo!(),
                Opcode::Anewarray => todo!(),
                Opcode::Arraylength => todo!(),
                Opcode::Athrow => todo!(),
                Opcode::Checkcast => {
                    let cpn = self.decode_u16();
                    let class_name = self.read_constant(self.read_constant(cpn).class()).utf8();
                    let class_name = self.runtime.read_static_string(class_name);
                    let class_name = class_name.to_string(); // TODO: bad
                    let id = self.runtime.load_class(&class_name)?;

                    let objectref = self.pop();
                    // TODO: check its type
                    self.push(objectref);
                }
                Opcode::Instanceof => {
                    let index = self.decode_u16();
                    let objectref = self.pop();
                    // TODO: check of objectref is a index
                    self.push(false);
                }
                Opcode::Monitorenter => todo!(),
                Opcode::Monitorexit => todo!(),
                Opcode::Wide => todo!(),
                Opcode::Multianewarray => todo!(),
                Opcode::GotoW => todo!(),
                Opcode::JsrW => todo!(),

                Opcode::Breakpoint |
                Opcode::ReservedFuture |
                Opcode::Impdep1 |
                Opcode::Impdep2 => return Err(RtError::ReservedInstruction)
            }
        }
        Ok(())
    }
    
    fn read_string_object(&self, sref: Value) -> Option<&str> {
        Some(match sref.into_ref() {
            Reference::Invalid => return None,
            Reference::Heap(offset) => {
                let len = self.heap[offset as usize / 4];
                let bytes = &self.heap.as_bytes_32aligned()[offset as usize + 4..][..len as usize];
                unsafe {
                    from_utf8_unchecked(bytes)
                }
            }
            Reference::Static(soffset) => {
                self.runtime.read_static_string(soffset)
            }
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ReturnCategory {
    Void = 0,
    Cat1 = 1,
    Cat2 = 2,
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Value(u32);
impl Value {
    pub const NULL: Self = Value(0);
    pub const ZERO: Self = Self::NULL;
    pub const fn into_u32(self) -> u32 {
        self.0
    }
    pub const fn into_u8(self) -> u8 {
        self.0 as u8
    }
    pub const fn into_u16(self) -> u16 {
        self.0 as u16
    }
    pub const fn into_i32(self) -> i32 {
        self.into_u32() as i32
    }
    pub const fn into_f32(self) -> f32 {
        f32::from_bits(self.into_u32())
    }
    pub const fn into_i8(self) -> i8 {
        self.into_u8() as i8
    }
    pub const fn into_i16(self) -> i16 {
        self.into_u16() as i16
    }
    pub const fn new_ref_static(n: u32) -> Self {
        Self(4 + n)
    }
    pub const fn new_ref_heap(n: u32) -> Self {
        Self(0x8000_0000 + n)
    }
    pub const fn into_ref(self) -> Reference {
        if self.0 < 4 {
            Reference::Invalid
        } else if self.0 < 0x8000_0000 {
            Reference::Static(self.0 - 4)
        } else {
            Reference::Heap(self.0 - 0x8000_0000)
        }
    }
    pub const fn offset(self, offset: u32) -> Value {
        Value(self.0 + offset)
    }
}
fn values_into_u64(value: (Value, Value)) -> u64 {
    unsafe { transmute(value) }
}
fn values_into_f64(value: (Value, Value)) -> f64 {
    f64::from_bits(values_into_u64(value))
}
fn i64_into_values(value: i64) -> (Value, Value) {
    unsafe { transmute(value) }
}
fn f64_into_values(value: f64) -> (Value, Value) {
    unsafe { transmute(value.to_bits()) }
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Reference {
    Invalid,
    Heap(u32),
    Static(u32),
}
impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value(value as u32)
    }
}
impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Value(value as i32 as u32)
    }
}
impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Value(value as i32 as u32)
    }
}
impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value(value as u32)
    }
}
impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value(value.to_bits())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RuntimeConstant(u32);
impl RuntimeConstant {
    pub const fn class(self) -> ConstIndex {
        self.0 as ConstIndex
    }
    pub const fn fieldref(self) -> (ConstIndex, ConstIndex) {
        unsafe { transmute(self.0) }
    }
    pub const fn methodref(self) -> (ConstIndex, ConstIndex) {
        unsafe { transmute(self.0) }
    }
    pub const fn interfacemethodref(self) -> (ConstIndex, ConstIndex) {
        unsafe { transmute(self.0) }
    }
    pub const fn string(self) -> ConstIndex {
        self.0 as ConstIndex
    }
    pub const fn integer(self) -> i32 {
        self.0 as i32
    }
    pub const fn float(self) -> f32 {
        f32::from_bits(self.0)
    }
    pub const fn nameandtype(self) -> (ConstIndex, ConstIndex) {
        unsafe { transmute(self.0) }
    }
    pub const fn utf8(self) -> u32 {
        self.0
    }
    pub const fn methodhandle(self) -> (u8, ConstIndex) {
        unsafe { transmute(self.0) }
    }
    pub const fn methodtype(self) -> ConstIndex {
        self.0 as ConstIndex
    }
    pub const fn invokedynamic(self) -> (ConstIndex, ConstIndex) {
        unsafe { transmute(self.0) }
    }
    pub const fn module(self) -> ConstIndex {
        self.0 as ConstIndex
    }pub const fn package(self) -> ConstIndex {
        self.0 as ConstIndex
    }
}
#[derive(Debug, Clone)]
enum RuntimeInfo {
    Builtin(Box<[fn(&mut RuntimeCtx)]>),
    Bytecode {
        method_code: Box<[BytecodeMethod]>,
        constant_pool: Box<[RuntimeConstant]>,
    }
}
mod member_table;
use self::member_table::MemberTable;
#[derive(Debug, Clone)]
struct LoadedClass {
    super_class: u32,
    interfaces: Box<[u32]>,
    /// name + ' ' + type -> offset (in instance fields, static fields or method table)
    member_table: MemberTable,
    static_fields: Box<Bytes32Aligned>,
    runtime_info: RuntimeInfo,
    data_size: u16,
}
#[derive(Debug, Clone, Copy)]
struct BytecodeMethod {
    max_stack: u16,
    max_locals: u16,
    arg_num: u16,
    code_location: usize,
}

impl LoadedClass {
    pub fn get_aligned_data_size(&self) -> u16 {
        (self.data_size + 3) & !3
    }
    fn member_table(&self) -> &MemberTable {
        &self.member_table
    }
    fn method(&self, method_id: u16) -> Result<BytecodeMethod, fn(&mut RuntimeCtx)> {
        match &self.runtime_info {
            RuntimeInfo::Builtin(method_code) => Err(method_code[method_id as usize]),
            RuntimeInfo::Bytecode{method_code, ..} => Ok(method_code[method_id as usize]),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Runtime {
    class_names: BTreeMap<Box<str>, u32>,
    classes: Vec<LoadedClass>,
    code: Vec<u32>,
    statics: Vec<u32>,
}
impl Runtime {
    pub fn new() -> Self {
        let mut class_names = BTreeMap::new();
        class_names.insert("java/lang/Object".into(), 0);
        Self {
            class_names,
            code: Vec::new(),
            statics: Vec::new(),
            classes: vec![LoadedClass {
                super_class: 0,
                data_size: 0,
                interfaces: Box::new([]),
                static_fields: Bytes32Aligned::new_zeroed(0),
                member_table: {
                    let mut table = MemberTable::new();
                    table.insert("<init>", MethodDescriptor::new_void([]), 0);
                    table.insert("equals", MethodDescriptor::new_void([]), 1);
                    table.insert("getClass", MethodDescriptor::new_ret([], FieldDescriptor::Boolean), 2);
                    table.insert("hashCode", MethodDescriptor::new_ret([], FieldDescriptor::Int), 3);
                    table.insert("toString", MethodDescriptor::new_ret([], FieldDescriptor::ClassRef("java/lang/String".into())), 4);
                    table
                },
                runtime_info: RuntimeInfo::Builtin(Box::new([
                    builtin_methods::obj_init,
                    builtin_methods::obj_equals,
                    builtin_methods::obj_get_class,
                    builtin_methods::obj_hash_code,
                    builtin_methods::obj_to_string,
                ])),
            }],
        }
    }
    fn get_class(&self, id: u32) -> &LoadedClass {
        &self.classes[id as usize]
    }
    /// The class path should separate packages with slashes (`/`)
    pub fn load_class(&mut self, classpath: &str) -> Result<u32> {
        if let Some(&v) = self.class_names.get(classpath) {
            return Ok(v);
        }
        eprintln!("Loading {classpath}");

        let id;
        if let Some(cls) = self.load_builtin(classpath) {
            id = self.classes.len() as u32;
            self.classes.push(cls);
        } else {
            let path: &Path = classpath.as_ref();
            let file = File::open(path.with_extension("class"))?;
            let class_file = ClassFile::from_reader(BufReader::new(file))?;
            id = self.load_class_file(&class_file)?;
        }
        self.class_names.insert(classpath.into(), id);
        Ok(id)
    }
    fn load_class_file(&mut self, class_file: &ClassFile) -> Result<u32> {
        let mut data_size = 0;
        let mut member_table = MemberTable::new();

        let super_class = self.load_class(class_file.constant_class(class_file.super_class).unwrap())?;
        let interfaces: Vec<_> = class_file.interfaces
            .iter()
            .map(|&i| {
                self.load_class(class_file.constant_class(i).unwrap())
            })
            .collect_result()?;

        for i in iter::once(super_class).chain(interfaces.iter().copied()) {
            let super_class = self.get_class(i);
            data_size += super_class.get_aligned_data_size();
        }
        let mut fields_ordered: Vec<_> = class_file.fields.iter().map(|field| {
            let name = class_file.constant_utf8(field.name_index).unwrap();
            let descriptor = class_file.constant_fdescriptor(field.descriptor_index).unwrap();
            let size: u16 = match descriptor {
                FieldDescriptor::Boolean => 1,
                FieldDescriptor::Byte => 1,
                FieldDescriptor::Char => 2,
                FieldDescriptor::Short => 2,
                FieldDescriptor::Int => 4,
                FieldDescriptor::Float => 4,
                FieldDescriptor::ClassRef(_) => 4,
                FieldDescriptor::ArrRef(_) => 4,
                FieldDescriptor::Double => 8,
                FieldDescriptor::Long => 8,
            };
            (name, size, descriptor, field.access_flags.contains(FieldAccess::STATIC))
        }).collect();
        fields_ordered.sort_by_key(|&(_, size, _, _)| Reverse(size));

        let mut offset = 0;
        let mut static_size = 0;
        for (name, size, d, is_static) in fields_ordered {
            if is_static {
                member_table.insert(name, d, static_size);
                static_size += size;
            } else {
                member_table.insert(name, d, offset);
                offset += size;
            }
        }
        data_size += offset;
        let mut method_code = Vec::with_capacity(class_file.methods.len());

        'wasd: for method in &class_file.methods {
            let name = class_file.constant_utf8(method.name_index).unwrap();
            let d = class_file.constant_mdescriptor(method.descriptor_index).unwrap();
            let id = method_code.len() as u16;
            member_table.insert(name, d.clone(), id);
            let implicit_this_arg = !method.access_flags.contains(MethodAccess::STATIC);

            for attrib in &method.attributes {
                match attrib {
                    &AttributeInfo::Code {
                        max_stack, max_locals, ref code, exception_table: _, attributes: _
                    } => {
                        let code_location = self.code.as_bytes_32aligned().len();
                        self.code.extend_from_bytes(&code.0);
                        method_code.push(BytecodeMethod {
                            max_stack,
                            max_locals,
                            arg_num: d.arg_types
                                .iter()
                                .map(|arg| arg.unit_size() as u16)
                                .chain(implicit_this_arg.then_some(1))
                                .sum(),
                            code_location,
                        });
                        continue 'wasd;
                    }
                    _ => (),
                }
            }
            unimplemented!();
        }

        let mut gap_val = None;
        let constant_pool = class_file.constant_pool
            .iter()
            .map(|c| {
                match *c {
                    Constant::Fieldref { class_index: a, name_and_type_index: b } |
                    Constant::Methodref { class_index: a, name_and_type_index: b } |
                    Constant::NameAndType { name_index: a, descriptor_index: b } |
                    Constant::Dynamic { bootstrap_method_attr_index: a, name_and_type_index: b } |
                    Constant::InvokeDynamic { bootstrap_method_attr_index: a, name_and_type_index: b } |
                    Constant::InterfaceMethodref { class_index: a, name_and_type_index: b } => unsafe {
                        RuntimeConstant(transmute((a, b)))
                    }
                    Constant::MethodHandle { reference_kind: a, reference_index: b } => unsafe {
                        RuntimeConstant(transmute((a, b)))
                    }
                    Constant::Class { name_index: a } |
                    Constant::MethodType { descriptor_index: a } |
                    Constant::Module { name_index: a } |
                    Constant::Package { name_index: a } |
                    Constant::String { string_index: a } => RuntimeConstant(a as u32),
                    Constant::Integer { bytes } |
                    Constant::Float { bytes } => RuntimeConstant(bytes),
                    Constant::Double { high_bytes, low_bytes } |
                    Constant::Long { high_bytes, low_bytes } => {
                        if cfg!(target_endian = "little") {
                            gap_val = Some(high_bytes);
                            RuntimeConstant(low_bytes)
                        } else {
                            gap_val = Some(high_bytes);
                            RuntimeConstant(low_bytes)
                        }
                    }
                    Constant::Utf8(ref s) => {
                        let offset = self.new_static_string_obj(s);
                        RuntimeConstant(offset)
                    }
                    Constant::Gap => RuntimeConstant(gap_val.take().unwrap())
                }
            })
            .collect();

        // load super class, first
        let loaded = LoadedClass {
            super_class,
            interfaces: interfaces.into_boxed_slice(),
            member_table,
            runtime_info: RuntimeInfo::Bytecode {
                method_code: method_code.into_boxed_slice(),
                constant_pool,
            },
            static_fields: Bytes32Aligned::new_zeroed((static_size as usize + 3) & !3),
            data_size,
        };
        let id = self.classes.len() as u32;
        self.classes.push(loaded);
        Ok(id)
    }
    pub fn new_static_string_obj(&mut self, s: &str) -> u32 {
        let string_location = self.statics.as_bytes_32aligned().len();
        self.statics.push(s.len() as u32);
        self.statics.extend_from_bytes(s.as_bytes());
        string_location as u32
    }
    pub fn new_static_obj(&mut self, data: &[u8]) -> u32 {
        let location = self.statics.as_bytes_32aligned().len();
        self.statics.extend_from_bytes(data);
        location as u32
    }
    pub fn read_static_string(&self, offset: u32) -> &str {
        let len = self.statics[offset as usize / 4];
        let bytes = &self.statics.as_bytes_32aligned()[offset as usize + 4..][..len as usize];
        unsafe {
            from_utf8_unchecked(bytes)
        }
    }
    pub fn run(&mut self, classpath: &str, args: Box<[Box<str>]>) -> Result<()> {
        let mut ctx = RuntimeCtx {
            frame_pointer: 0,
            pc: 0,
            max_locals: 0,
            cur_class: 0,
            stack: Vec::new(),
            heap: Vec::new(),
            return_stack: Vec::with_capacity(1),
            runtime: self,
        };
        // TODO: generate a real array to push to the stack
        ctx.push(Value(args.as_ptr() as usize as u32));
        ctx.call_named(classpath, "main", MethodDescriptor::new_void([
            FieldDescriptor::ArrRef(Box::new(FieldDescriptor::ClassRef("java/lang/String".into())))
        ]))?;
        ctx.run_inner()
    }
    fn load_builtin(&mut self, classpath: &str) -> Option<LoadedClass> {
        use self::FieldDescriptor::*;
        Some(match classpath {
            "java/lang/String" => LoadedClass {
                super_class: 0,
                interfaces: Box::new([]),
                static_fields: Bytes32Aligned::new_zeroed(0),
                data_size: u16::MAX,
                member_table: MemberTable::new(),
                runtime_info: RuntimeInfo::Builtin(Box::new([])),
            },
            "java/lang/System" => LoadedClass {
                super_class: 0,
                interfaces: Box::new([]),
                static_fields: {
                    let mut fields = Bytes32Aligned::new_zeroed(4*3);
                    for (stream, i) in fields.as_u32_slice_mut().iter_mut().zip(0..) {
                        *stream = Value::new_ref_static(self.new_static_obj(&[i])).into_u32();
                    }
                    fields
                },
                data_size: 0,
                member_table: {
                    let mut table = MemberTable::new();
                    table.insert("out", ClassRef("java/io/PrintStream".into()),  0);
                    table
                },
                runtime_info: RuntimeInfo::Builtin(Box::new([])),
            },
            "java/io/PrintStream" => LoadedClass {
                super_class: 0,
                interfaces: Box::new([]),
                static_fields: Bytes32Aligned::new_zeroed(0),
                data_size: 1,
                member_table: {
                    let mut table = MemberTable::new();
                    table.insert("println", MethodDescriptor::new_void([ClassRef("java/lang/String".into())]), 0);
                    table
                },
                runtime_info: RuntimeInfo::Builtin(Box::new([
                    builtin_methods::printstream_println_str,
                ])),
            },
            _ => return None,
        })
    }
}

#[derive(Debug)]
pub enum RtError {
    Io(io::Error),
    Descriptor(DescriptorError),
    ReservedInstruction,
}

impl From<io::Error> for RtError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<DescriptorError> for RtError {
    fn from(e: DescriptorError) -> Self {
        Self::Descriptor(e)
    }
}
