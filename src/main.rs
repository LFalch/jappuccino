use std::{
    env::args_os,
    fs::File,
    io::{BufReader, Write, stdout},
};

use jappuccino::class::{AttributeInfo, ClassFile, Constant, ExceptionEntry, Field, LineNumberEntry, Method};

fn main() {
    for arg in args_os().skip(1) {
        stdout().write_all(arg.as_encoded_bytes()).unwrap();
        println!(":");
        let file = File::open(arg).unwrap();
        let class = ClassFile::from_reader(BufReader::new(file)).unwrap();
        print_class(class);
    }
}

fn print_class(class: ClassFile) {
    let ClassFile {
        version,
        constant_pool,
        access_flags,
        this_class,
        super_class,
        interfaces,
        fields,
        methods,
        attributes,
    } = class;

    println!("version: {}.{}", version.0, version.1);
    println!("constant pool:");
    for (constant, n) in constant_pool.iter().zip(1..) {
        print!("  {n}: ");
        match *constant {
            Constant::Class { name_index } => println!("Class name = {name_index}"),
            Constant::Fieldref {
                class_index,
                name_and_type_index,
            } => println!("Fieldref class = {class_index} name_and_type = {name_and_type_index}"),
            Constant::Methodref {
                class_index,
                name_and_type_index,
            } => println!("Methodref class = {class_index} name_and_type = {name_and_type_index}"),
            Constant::InterfaceMethodref {
                class_index,
                name_and_type_index,
            } => println!(
                "InterfaceMethodref class = {class_index} name_and_type = {name_and_type_index}"
            ),
            Constant::String { string_index } => println!("String {string_index}"),
            Constant::Integer { bytes } => println!("Integer = {bytes}"),
            Constant::Float { bytes } => println!("Float = {bytes:04x}"),
            Constant::Long {
                high_bytes,
                low_bytes,
            } => println!("Long name = {high_bytes:04x} {low_bytes:04x}"),
            Constant::Double {
                high_bytes,
                low_bytes,
            } => println!("Double name = {high_bytes:04x} {low_bytes:04x}"),
            Constant::NameAndType {
                name_index,
                descriptor_index,
            } => println!("NameAndType name = {name_index} descriptor = {descriptor_index}"),
            Constant::Utf8(ref s) => println!("{s:?}"),
            Constant::MethodHandle {
                reference_kind,
                reference_index,
            } => println!(
                "MethodHandle reference_kind = {reference_kind}, reference = {reference_index}"
            ),
            Constant::MethodType { descriptor_index } => {
                println!("MethodType descriptor = {descriptor_index}")
            }
            Constant::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => println!(
                "InvokeDynamic bootstrap_method_attr = {bootstrap_method_attr_index} name_and_type = {name_and_type_index}"
            ),
            Constant::Module { name_index } => println!("Module {name_index}"),
            Constant::Package { name_index } => println!("Package {name_index}"),
        }
    }
    println!("access flags: {access_flags}");
    println!("this: {this_class}");
    println!("super: {super_class}");
    println!("interfaces:");
    for (n, interface) in interfaces.into_iter().enumerate() {
        println!("  {n}: const {interface}");
    }
    println!("fields:");
    for (n, field) in fields.into_iter().enumerate() {
        print!("  {n}: ");
        let Field {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        } = field;
        print!("access_flags = {access_flags}");
        print!("name = {name_index}");
        print!("descriptor = {descriptor_index}");
        print_attributes(&attributes, 2);
    }
    println!("methods:");
    for (n, method) in methods.into_iter().enumerate() {
        print!("  {n}: ");
        let Method {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        } = method;
        print!("access_flags = {access_flags}");
        print!("name = {name_index}");
        print!("descriptor = {descriptor_index}");

        print_attributes(&attributes, 2);
    }
    println!("attributes:");
    print_attributes(&attributes, 1);
}

fn print_attributes(attributes: &[AttributeInfo], indent: u8) {
    for (n, attribute) in attributes.into_iter().enumerate() {
        for _ in 0..indent {
            print!("  ");
        }
        print!("{n}: ");
        match attribute {
            AttributeInfo::ConstantValue { constantvalue_index } => {
                println!("ConstantValue {constantvalue_index}");
            }
            AttributeInfo::Code { max_stack, max_locals, code, exception_table, attributes } => {
                println!("Code:");
                for _ in 0..indent+1 {
                    print!("  ");
                }
                println!("max_stack = {max_stack} max_locals = {max_locals}");
                for _ in 0..indent+1 {
                    print!("  ");
                }
                println!("code = {code:?}"); // TODO: show code as mnemonics
                for _ in 0..indent+1 {
                    print!("  ");
                }
                println!("exception_table:");
                for &ExceptionEntry { start_pc, end_pc, handler_pc, catch_type } in exception_table {
                    for _ in 0..indent+2 {
                        print!("  ");
                    }
                    println!("start_pc = {start_pc} end_pc = {end_pc} handler_pc {handler_pc} catch_type = const {catch_type}");
                }
                for _ in 0..indent+1 {
                    print!("  ");
                }
                println!("attributes:");
                print_attributes(attributes, indent+2);
            }
            AttributeInfo::SourceFile { sourcefile_index } => {
                println!("SourceFile {sourcefile_index}");
            }
            AttributeInfo::LineNumberTable(items) => {
                println!("LineNumberTable:");
                for item in items {
                    let &LineNumberEntry { start_pc, line_number } = item;
                    for _ in 0..indent+1 {
                        print!("  ");
                    }
                    println!("start_pc = {start_pc} line_number {line_number}");
                }
            }
            AttributeInfo::Unknown(name, raw_bytes) => {
                println!("{name} = {raw_bytes:?}");
            }
        }
    }
}
