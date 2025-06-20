use std::{
    env::args_os,
    fs::File,
    io::{BufReader, Write, stdout},
};

use jappuccino::class::{AttributeInfo, ClassFile, ConstIndex, Constant, ExceptionEntry, Field, LineNumberEntry, Method};

fn main() {
    let mut show_constant_pool = false;
    for arg in args_os().skip(1) {
        if arg == "-c" {
            show_constant_pool = true;
            continue;
        }
        stdout().write_all(arg.as_encoded_bytes()).unwrap();
        println!(":");
        let file = File::open(arg).unwrap();
        let class = ClassFile::from_reader(BufReader::new(file)).unwrap();
        print_class(class, show_constant_pool);
    }
}

fn print_class(class: ClassFile, show_constant_pool: bool) {
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
    if show_constant_pool {
        println!("constant pool:");
        for (constant, n) in constant_pool.iter().zip(1..) {
            print!("  {n}: ");
            print_constant(constant, &constant_pool);
            println!();
        }
    }
    println!("access flags: {access_flags}");
    print!("this: ");
    print_cpn(this_class, &constant_pool);
    println!();
    print!("super: ");
    print_cpn(super_class, &constant_pool);
    println!();
    println!("interfaces:");
    for (n, interface) in interfaces.into_iter().enumerate() {
        print!("  {n}: ");
        print_cpn(interface, &constant_pool);
        println!();
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
        print!("{access_flags} ");
        print_cpn(name_index, &constant_pool);
        print!(" ");
        print_cpn(descriptor_index, &constant_pool);
        println!();
        print_attributes(&attributes, 2, &constant_pool);
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
        print!("{access_flags} ");
        print_cpn(name_index, &constant_pool);
        print!(" ");
        print_cpn(descriptor_index, &constant_pool);
        println!();
        print_attributes(&attributes, 2, &constant_pool);
    }
    println!("attributes:");
    print_attributes(&attributes, 1, &constant_pool);
}

fn get_constant(n: ConstIndex, constant_pool: &[Constant]) -> &Constant {
    &constant_pool[n as usize - 1]
}
#[inline]
fn print_cpn(n: ConstIndex, constant_pool: &[Constant]) {
    print!("#{n} ");
    print_constant(get_constant(n, constant_pool), constant_pool);
}
fn print_constant(constant: &Constant, constant_pool: &[Constant]) {
    match *constant {
        Constant::Class { name_index } => {
            print!("Class ");
            print_cpn(name_index, constant_pool);
        }
        Constant::Fieldref {
            class_index,
            name_and_type_index,
        } => {
            print!("Fieldref class = ");
            print_cpn(class_index, constant_pool);
            print!(" name_and_type = ");
            print_cpn(name_and_type_index, constant_pool);
        }
        Constant::Methodref {
            class_index,
            name_and_type_index,
        } => {
            print!("Methodref class = ");
            print_cpn(class_index, constant_pool);
            print!(" name_and_type = ");
            print_cpn(name_and_type_index, constant_pool);
        }
        Constant::InterfaceMethodref {
            class_index,
            name_and_type_index,
        } => {
            print!(
            "InterfaceMethodref class = ");
            print_cpn(class_index, constant_pool);
            print!(" name_and_type = ");
            print_cpn(name_and_type_index, constant_pool);
        },
        Constant::String { string_index } => {
            print!("String ");
            print_cpn(string_index, constant_pool);
        }
        Constant::Integer { bytes } => print!("Integer = {bytes}"),
        Constant::Float { bytes } => print!("Float = {bytes:04x}"),
        Constant::Long {
            high_bytes,
            low_bytes,
        } => print!("Long = {high_bytes:04x} {low_bytes:04x}"),
        Constant::Double {
            high_bytes,
            low_bytes,
        } => print!("Double = {high_bytes:04x} {low_bytes:04x}"),
        Constant::NameAndType {
            name_index,
            descriptor_index,
        } => {
            print!("NameAndType name = ");
            print_cpn(name_index, constant_pool);
            print!(" descriptor = ");
            print_cpn(descriptor_index, constant_pool);
        },
        Constant::Utf8(ref s) => print!("{s:?}"),
        Constant::MethodHandle {
            reference_kind,
            reference_index,
        } => {
            print!("MethodHandle reference_kind = {reference_kind} reference = ");
            print_cpn(reference_index, constant_pool);
        }
        Constant::MethodType { descriptor_index } => {
            print!("MethodType descriptor = ");
            print_cpn(descriptor_index, constant_pool);
        }
        Constant::InvokeDynamic {
            bootstrap_method_attr_index,
            name_and_type_index,
        } => println!(
            "InvokeDynamic bootstrap_method_attr = {bootstrap_method_attr_index} name_and_type = {name_and_type_index}"
        ),
        Constant::Module { name_index } => {
            print!("Module ");
            print_cpn(name_index, constant_pool);
        },
        Constant::Package { name_index } => {
            print!("Package ");
            print_cpn(name_index, constant_pool);
        },
    }
}

fn print_attributes(attributes: &[AttributeInfo], indent: u8, constant_pool: &[Constant]) {
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
                print_attributes(attributes, indent+2, constant_pool);
            }
            &AttributeInfo::SourceFile { sourcefile_index } => {
                print!("SourceFile ");
                print_cpn(sourcefile_index, constant_pool);
                println!();
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
