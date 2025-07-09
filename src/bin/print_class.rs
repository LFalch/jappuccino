use std::{
    env::args_os,
    fs::File,
    io::{BufReader, Write, stdout},
};

use jappuccino::{
    class::{display_constant, display_descriptor, display_field_descriptor, display_method_descriptor, AttributeInfo, ClassFile, Constant, ExceptionEntry, Field, LineNumberEntry, LocalVariableEntry, Method}, code::display_code,
    // descriptor::{AnyDescriptor, FieldDescriptor, MethodDescriptor},
};

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
        print_classfile(&class, show_constant_pool);
    }
}

fn print_classfile(class: &ClassFile, show_constant_pool: bool) {
    let &ClassFile {
        version,
        ref constant_pool,
        access_flags,
        this_class,
        super_class,
        ref interfaces,
        ref fields,
        ref methods,
        ref attributes,
    } = class;

    println!("version: {}.{}", version.0, version.1);
    if show_constant_pool {
        println!("constant pool:");
        for n in 1..=constant_pool.len() as u16 {
            println!("  {n}: {}", display_constant(n, &constant_pool));
        }
    }
    println!("access flags: {access_flags}");
    println!("this: {}", display_constant(this_class, &constant_pool));
    println!("super: {}", display_constant(super_class, &constant_pool));
    println!("interfaces:");
    for (n, &interface) in interfaces.iter().enumerate() {
        println!("  {n}: {}", display_constant(interface, &constant_pool));
    }
    println!("fields:");
    for (n, field) in fields.into_iter().enumerate() {
        print!("  {n}: ");
        let &Field {
            access_flags,
            name_index,
            descriptor_index,
            ref attributes,
        } = field;
        println!("{access_flags} {} {}", display_field_descriptor(descriptor_index, &constant_pool), display_constant(name_index, &constant_pool));
        print_attributes(attributes, 2, &constant_pool);
    }
    println!("methods:");
    for (n, method) in methods.into_iter().enumerate() {
        print!("  {n}: ");
        let &Method {
            access_flags,
            name_index,
            descriptor_index,
            ref attributes,
        } = method;
        println!("{access_flags} {}", display_method_descriptor(name_index, descriptor_index, constant_pool));
        print_attributes(attributes, 2, &constant_pool);
    }
    println!("attributes:");
    print_attributes(&attributes, 1, &constant_pool);
}

fn print_attributes(attributes: &[AttributeInfo], indent: u8, constant_pool: &[Constant]) {
    for (n, attribute) in attributes.into_iter().enumerate() {
        for _ in 0..indent {
            print!("  ");
        }
        print!("{n}: ");
        match attribute {
            &AttributeInfo::ConstantValue { constantvalue_index } => {
                println!("ConstantValue {}", display_constant(constantvalue_index, constant_pool));
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
                println!("code:");
                print!("{}", display_code(&code.0, constant_pool, 2 * (indent as usize + 2)));
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
                println!("SourceFile {}", display_constant(sourcefile_index, constant_pool));
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
            AttributeInfo::LocalVariableTable(entries) => {
                println!("LocalVariableTable:");
                for entry in entries {
                    let &LocalVariableEntry { start_pc, length, name_index, descriptor_index, index } = entry;
                    for _ in 0..indent+1 {
                        print!("  ");
                    }
                    println!("{} => {index} @ Code[{start_pc}..{}] ", display_descriptor(name_index, descriptor_index, constant_pool), start_pc+length);
                }
            }
            AttributeInfo::StackMapTable(entries) => {
                println!("StackMapTable:");
                for entry in entries {
                    for _ in 0..indent+1 {
                        print!("  ");
                    }
                    println!("{entry:?}");
                }
            }
            AttributeInfo::Exceptions(raw_bytes) => println!("Exceptions {raw_bytes:?}"),
            AttributeInfo::InnerClasses(raw_bytes) => println!("InnerClasses {raw_bytes:?}"),
            AttributeInfo::EnclosingMethod(raw_bytes) => println!("EnclosingMethod {raw_bytes:?}"),
            AttributeInfo::Synthetic(raw_bytes) => println!("Synthetic {raw_bytes:?}"),
            AttributeInfo::Signature(raw_bytes) => println!("Signature {raw_bytes:?}"),
            AttributeInfo::SourceDebugExtension(raw_bytes) => println!("SourceDebugExtension {raw_bytes:?}"),
            AttributeInfo::LocalVariableTypeTable(raw_bytes) => println!("LocalVariableTypeTable {raw_bytes:?}"),
            AttributeInfo::Deprecated(raw_bytes) => println!("Deprecated {raw_bytes:?}"),
            AttributeInfo::RuntimeVisibleAnnotations(raw_bytes) => println!("RuntimeVisibleAnnotations {raw_bytes:?}"),
            AttributeInfo::RuntimeInvisibleAnnotations(raw_bytes) => println!("RuntimeInvisibleAnnotations {raw_bytes:?}"),
            AttributeInfo::RuntimeVisibleParameterAnnotations(raw_bytes) => println!("RuntimeVisibleParameterAnnotations {raw_bytes:?}"),
            AttributeInfo::RuntimeInvisibleParameterAnnotations(raw_bytes) => println!("RuntimeInvisibleParameterAnnotations {raw_bytes:?}"),
            AttributeInfo::RuntimeVisibleTypeAnnotations(raw_bytes) => println!("RuntimeVisibleTypeAnnotations {raw_bytes:?}"),
            AttributeInfo::RuntimeInvisibleTypeAnnotations(raw_bytes) => println!("RuntimeInvisibleTypeAnnotations {raw_bytes:?}"),
            AttributeInfo::AnnotationDefault(raw_bytes) => println!("AnnotationDefault {raw_bytes:?}"),
            AttributeInfo::BootstrapMethods(raw_bytes) => println!("BootstrapMethods {raw_bytes:?}"),
            AttributeInfo::NestHost(raw_bytes) => println!("NestHost {raw_bytes:?}"),
            AttributeInfo::NestMembers(raw_bytes) => println!("NestMembers {raw_bytes:?}"),
            AttributeInfo::PermittedSubclasses(raw_bytes) => println!("PermittedSubclasses {raw_bytes:?}"),
            AttributeInfo::MethodParameters(raw_bytes) => println!("MethodParameters {raw_bytes:?}"),
            AttributeInfo::Module(raw_bytes) => println!("Module {raw_bytes:?}"),
            AttributeInfo::ModulePackages(raw_bytes) => println!("ModulePackages {raw_bytes:?}"),
            AttributeInfo::ModuleMainClass(raw_bytes) => println!("ModuleMainClass {raw_bytes:?}"),
        }
    }
}
