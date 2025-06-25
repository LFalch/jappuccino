use num_enum::{FromPrimitive, IntoPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum Opcode {
    Nop = 0x0,
    AconstNull = 0x1,
    IconstM1 = 0x2,
    Iconst0 = 0x3,
    Iconst1 = 0x4,
    Iconst2 = 0x5,
    Iconst3 = 0x6,
    Iconst4 = 0x7,
    Iconst5 = 0x8,
    Lconst0 = 0x9,
    Lconst1 = 0xa,
    Fconst0 = 0xb,
    Fconst1 = 0xc,
    Fconst2 = 0xd,
    Dconst0 = 0xe,
    Dconst1 = 0xf,

    Bipush = 0x10,
    Sipush = 0x11,
    Ldc = 0x12,
    LdcW = 0x13,
    Ldc2W = 0x14,
    Iload = 0x15,
    Lload = 0x16,
    Fload = 0x17,
    Dload = 0x18,
    Aload = 0x19,
    Iload0 = 0x1a,
    Iload1 = 0x1b,
    Iload2 = 0x1c,
    Iload3 = 0x1d,
    Lload0 = 0x1e,
    Lload1 = 0x1f,

    Lload2 = 0x20,
    Lload3 = 0x21,
    Fload0 = 0x22,
    Fload1 = 0x23,
    Fload2 = 0x24,
    Fload3 = 0x25,
    Dload0 = 0x26,
    Dload1 = 0x27,
    Dload2 = 0x28,
    Dload3 = 0x29,
    Aload0 = 0x2a,
    Aload1 = 0x2b,
    Aload2 = 0x2c,
    Aload3 = 0x2d,
    Iaload = 0x2e,
    Laload = 0x2f,

    Faload = 0x30,
    Daload = 0x31,
    Aaload = 0x32,
    Baload = 0x33,
    Caload = 0x34,
    Saload = 0x35,
    Istore = 0x36,
    Lstore = 0x37,
    Fstore = 0x38,
    Dstore = 0x39,
    Astore = 0x3a,
    Istore0 = 0x3b,
    Istore1 = 0x3c,
    Istore2 = 0x3d,
    Istore3 = 0x3e,
    Lstore0 = 0x3f,

    Lstore1 = 0x40,
    Lstore2 = 0x41,
    Lstore3 = 0x42,
    Fstore0 = 0x43,
    Fstore1 = 0x44,
    Fstore2 = 0x45,
    Fstore3 = 0x46,
    Dstore0 = 0x47,
    Dstore1 = 0x48,
    Dstore2 = 0x49,
    Dstore3 = 0x4a,
    Astore0 = 0x4b,
    Astore1 = 0x4c,
    Astore2 = 0x4d,
    Astore3 = 0x4e,
    Iastore = 0x4f,

    Lastore = 0x50,
    Fastore = 0x51,
    Dastore = 0x52,
    Aastore = 0x53,
    Bastore = 0x54,
    Castore = 0x55,
    Sastore = 0x56,
    Pop = 0x57,
    Pop2 = 0x58,
    Dup = 0x59,
    DupX1 = 0x5a,
    DupX2 = 0x5b,
    Dup2 = 0x5c,
    Dup2X1 = 0x5d,
    Dup2X2 = 0x5e,
    Swap = 0x5f,

    Iadd = 0x60,
    Ladd = 0x61,
    Fadd = 0x62,
    Dadd = 0x63,
    Isub = 0x64,
    Lsub = 0x65,
    Fsub = 0x66,
    Dsub = 0x67,
    Imul = 0x68,
    Lmul = 0x69,
    Fmul = 0x6a,
    Dmul = 0x6b,
    Idiv = 0x6c,
    Ldiv = 0x6d,
    Fdiv = 0x6e,
    Ddiv = 0x6f,

    Irem = 0x70,
    Lrem = 0x71,
    Frem = 0x72,
    Drem = 0x73,
    Ineg = 0x74,
    Lneg = 0x75,
    Fneg = 0x76,
    Dneg = 0x77,
    Ishl = 0x78,
    Lshl = 0x79,
    Ishr = 0x7a,
    Lshr = 0x7b,
    Iushr = 0x7c,
    Lushr = 0x7d,
    Iand = 0x7e,
    Land = 0x7f,

    Ior = 0x80,
    Lor = 0x81,
    Ixor = 0x82,
    Lxor = 0x83,
    Iinc = 0x84,
    I2l = 0x85,
    I2f = 0x86,
    I2d = 0x87,
    L2i = 0x88,
    L2f = 0x89,
    L2d = 0x8a,
    F2i = 0x8b,
    F2l = 0x8c,
    F2d = 0x8d,
    D2i = 0x8e,
    D2l = 0x8f,

    D2f = 0x90,
    I2b = 0x91,
    I2c = 0x92,
    I2s = 0x93,
    Lcmp = 0x94,
    Fcmpl = 0x95,
    Fcmpg = 0x96,
    Dcmpl = 0x97,
    Dcmpg = 0x98,
    Ifeq = 0x99,
    Ifne = 0x9a,
    Iflt = 0x9b,
    Ifge = 0x9c,
    Ifgt = 0x9d,
    Ifle = 0x9e,
    IfIcmpeq = 0x9f,

    IfIcmpne = 0xa0,
    IfIcmplt = 0xa1,
    IfIcmpge = 0xa2,
    IfIcmpgt = 0xa3,
    IfIcmple = 0xa4,
    IfAcmpeq = 0xa5,
    IfAcmpne = 0xa6,
    Goto = 0xa7,
    Jsr = 0xa8,
    Ret = 0xa9,
    Tableswitch = 0xaa,
    Lookupswitch = 0xab,
    Ireturn = 0xac,
    Lreturn = 0xad,
    Freturn = 0xae,
    Dreturn = 0xaf,

    Areturn = 0xb0,
    Return = 0xb1,
    Getstatic = 0xb2,
    Putstatic = 0xb3,
    Getfield = 0xb4,
    Putfield = 0xb5,
    Invokevirtual = 0xb6,
    Invokespecial = 0xb7,
    Invokestatic = 0xb8,
    Invokeinterface = 0xb9,
    Invokedynamic = 0xba,
    New = 0xbb,
    Newarray = 0xbc,
    Anewarray = 0xbd,
    Arraylength = 0xbe,
    Athrow = 0xbf,

    Checkcast = 0xc0,
    Instanceof = 0xc1,
    Monitorenter = 0xc2,
    Monitorexit = 0xc3,
    Wide = 0xc4,
    Multianewarray = 0xc5,
    Ifnull = 0xc6,
    Ifnonnull = 0xc7,
    GotoW = 0xc8,
    JsrW = 0xc9,

/// RESERVED
    Breakpoint = 0xca,
    #[num_enum(default)]
    ReservedFuture,

    Impdep1 = 0xfe,
    Impdep2 = 0xff,
}

impl Opcode {
    pub const fn immediates(self) -> u8 {
        match self {
            Opcode::Nop |
            Opcode::AconstNull |
            Opcode::IconstM1 |
            Opcode::Iconst0 |
            Opcode::Iconst1 |
            Opcode::Iconst2 |
            Opcode::Iconst3 |
            Opcode::Iconst4 |
            Opcode::Iconst5 |
            Opcode::Lconst0 |
            Opcode::Lconst1 |
            Opcode::Fconst0 |
            Opcode::Fconst1 |
            Opcode::Fconst2 |
            Opcode::Dconst0 |
            Opcode::Dconst1 |
            Opcode::Iload0 |
            Opcode::Iload1 |
            Opcode::Iload2 |
            Opcode::Iload3 |
            Opcode::Lload0 |
            Opcode::Lload1 |
            Opcode::Lload2 |
            Opcode::Lload3 |
            Opcode::Fload0 |
            Opcode::Fload1 |
            Opcode::Fload2 |
            Opcode::Fload3 |
            Opcode::Dload0 |
            Opcode::Dload1 |
            Opcode::Dload2 |
            Opcode::Dload3 |
            Opcode::Aload0 |
            Opcode::Aload1 |
            Opcode::Aload2 |
            Opcode::Aload3 |
            Opcode::Iaload |
            Opcode::Laload |
            Opcode::Faload |
            Opcode::Daload |
            Opcode::Aaload |
            Opcode::Baload |
            Opcode::Caload |
            Opcode::Saload |
            Opcode::Istore0 |
            Opcode::Istore1 |
            Opcode::Istore2 |
            Opcode::Istore3 |
            Opcode::Lstore0 |
            Opcode::Lstore1 |
            Opcode::Lstore2 |
            Opcode::Lstore3 |
            Opcode::Fstore0 |
            Opcode::Fstore1 |
            Opcode::Fstore2 |
            Opcode::Fstore3 |
            Opcode::Dstore0 |
            Opcode::Dstore1 |
            Opcode::Dstore2 |
            Opcode::Dstore3 |
            Opcode::Astore0 |
            Opcode::Astore1 |
            Opcode::Astore2 |
            Opcode::Astore3 |
            Opcode::Iastore |
            Opcode::Lastore |
            Opcode::Fastore |
            Opcode::Dastore |
            Opcode::Aastore |
            Opcode::Bastore |
            Opcode::Castore |
            Opcode::Sastore |
            Opcode::Pop |
            Opcode::Pop2 |
            Opcode::Dup |
            Opcode::DupX1 |
            Opcode::DupX2 |
            Opcode::Dup2 |
            Opcode::Dup2X1 |
            Opcode::Dup2X2 |
            Opcode::Swap |
            Opcode::Iadd |
            Opcode::Ladd |
            Opcode::Fadd |
            Opcode::Dadd |
            Opcode::Isub |
            Opcode::Lsub |
            Opcode::Fsub |
            Opcode::Dsub |
            Opcode::Imul |
            Opcode::Lmul |
            Opcode::Fmul |
            Opcode::Dmul |
            Opcode::Idiv |
            Opcode::Ldiv |
            Opcode::Fdiv |
            Opcode::Ddiv |
            Opcode::Irem |
            Opcode::Lrem |
            Opcode::Frem |
            Opcode::Drem |
            Opcode::Ineg |
            Opcode::Lneg |
            Opcode::Fneg |
            Opcode::Dneg |
            Opcode::Ishl |
            Opcode::Lshl |
            Opcode::Ishr |
            Opcode::Lshr |
            Opcode::Iushr |
            Opcode::Lushr |
            Opcode::Iand |
            Opcode::Land |
            Opcode::Ior |
            Opcode::Lor |
            Opcode::Ixor |
            Opcode::Lxor |
            Opcode::I2l |
            Opcode::I2f |
            Opcode::I2d |
            Opcode::L2i |
            Opcode::L2f |
            Opcode::L2d |
            Opcode::F2i |
            Opcode::F2l |
            Opcode::F2d |
            Opcode::D2i |
            Opcode::D2l |
            Opcode::D2f |
            Opcode::I2b |
            Opcode::I2c |
            Opcode::I2s |
            Opcode::Lcmp |
            Opcode::Fcmpl |
            Opcode::Fcmpg |
            Opcode::Dcmpl |
            Opcode::Dcmpg |
            Opcode::Ireturn |
            Opcode::Lreturn |
            Opcode::Freturn |
            Opcode::Dreturn |
            Opcode::Areturn |
            Opcode::Return |
            Opcode::Arraylength |
            Opcode::Athrow |
            Opcode::Monitorenter |
            Opcode::Monitorexit => 0,

            Opcode::Bipush |
            Opcode::Ldc |
            Opcode::Iload |
            Opcode::Lload |
            Opcode::Fload |
            Opcode::Dload |
            Opcode::Aload |
            Opcode::Istore |
            Opcode::Lstore |
            Opcode::Fstore |
            Opcode::Dstore |
            Opcode::Astore |
            Opcode::Ret |
            Opcode::Newarray => 1,
            Opcode::Sipush |
            Opcode::LdcW |
            Opcode::Ldc2W |
            Opcode::Iinc |
            Opcode::Ifeq |
            Opcode::Ifne |
            Opcode::Iflt |
            Opcode::Ifge |
            Opcode::Ifgt |
            Opcode::Ifle |
            Opcode::IfIcmpeq |
            Opcode::IfIcmpne |
            Opcode::IfIcmplt |
            Opcode::IfIcmpge |
            Opcode::IfIcmpgt |
            Opcode::IfIcmple |
            Opcode::IfAcmpeq |
            Opcode::IfAcmpne |
            Opcode::Goto |
            Opcode::Jsr |
            Opcode::Getstatic |
            Opcode::Putstatic |
            Opcode::Getfield |
            Opcode::Putfield |
            Opcode::Invokevirtual |
            Opcode::Invokespecial |
            Opcode::Invokestatic |
            Opcode::New |
            Opcode::Anewarray |
            Opcode::Checkcast |
            Opcode::Instanceof |
            Opcode::Ifnull |
            Opcode::Ifnonnull => 2,
            Opcode::Wide => 3 /* or 5 with iinc */,
            Opcode::Multianewarray => 3,
            Opcode::Invokeinterface => 4,
            Opcode::Invokedynamic => 4,
            Opcode::GotoW => 4,
            Opcode::JsrW => 4,
            Opcode::Tableswitch => 16 /* + */,
            Opcode::Lookupswitch => 8 /* + */,

            Opcode::Breakpoint => 0,
            Opcode::ReservedFuture => 0,
            Opcode::Impdep1 => 0,
            Opcode::Impdep2 => 0,
        }
    }
    pub const fn mnemonic(self) -> &'static str {
        match self {
            Opcode::Nop => "nop",
            Opcode::AconstNull => "aconst_null",
            Opcode::IconstM1 => "iconst_m1",
            Opcode::Iconst0 => "iconst_0",
            Opcode::Iconst1 => "iconst_1",
            Opcode::Iconst2 => "iconst_2",
            Opcode::Iconst3 => "iconst_3",
            Opcode::Iconst4 => "iconst_4",
            Opcode::Iconst5 => "iconst_5",
            Opcode::Lconst0 => "lconst_0",
            Opcode::Lconst1 => "lconst_1",
            Opcode::Fconst0 => "fconst_0",
            Opcode::Fconst1 => "fconst_1",
            Opcode::Fconst2 => "fconst_2",
            Opcode::Dconst0 => "dconst_0",
            Opcode::Dconst1 => "dconst_1",
            Opcode::Bipush => "bipush",
            Opcode::Sipush => "sipush",
            Opcode::Ldc => "ldc",
            Opcode::LdcW => "ldc_w",
            Opcode::Ldc2W => "ldc2_w",
            Opcode::Iload => "iload",
            Opcode::Lload => "lload",
            Opcode::Fload => "fload",
            Opcode::Dload => "dload",
            Opcode::Aload => "aload",
            Opcode::Iload0 => "iload_0",
            Opcode::Iload1 => "iload_1",
            Opcode::Iload2 => "iload_2",
            Opcode::Iload3 => "iload_3",
            Opcode::Lload0 => "lload_0",
            Opcode::Lload1 => "lload_1",
            Opcode::Lload2 => "lload_2",
            Opcode::Lload3 => "lload_3",
            Opcode::Fload0 => "fload_0",
            Opcode::Fload1 => "fload_1",
            Opcode::Fload2 => "fload_2",
            Opcode::Fload3 => "fload_3",
            Opcode::Dload0 => "dload_0",
            Opcode::Dload1 => "dload_1",
            Opcode::Dload2 => "dload_2",
            Opcode::Dload3 => "dload_3",
            Opcode::Aload0 => "aload_0",
            Opcode::Aload1 => "aload_1",
            Opcode::Aload2 => "aload_2",
            Opcode::Aload3 => "aload_3",
            Opcode::Iaload => "iaload",
            Opcode::Laload => "laload",
            Opcode::Faload => "faload",
            Opcode::Daload => "daload",
            Opcode::Aaload => "aaload",
            Opcode::Baload => "baload",
            Opcode::Caload => "caload",
            Opcode::Saload => "saload",
            Opcode::Istore => "istore",
            Opcode::Lstore => "lstore",
            Opcode::Fstore => "fstore",
            Opcode::Dstore => "dstore",
            Opcode::Astore => "astore",
            Opcode::Istore0 => "istore_0",
            Opcode::Istore1 => "istore_1",
            Opcode::Istore2 => "istore_2",
            Opcode::Istore3 => "istore_3",
            Opcode::Lstore0 => "lstore_0",
            Opcode::Lstore1 => "lstore_1",
            Opcode::Lstore2 => "lstore_2",
            Opcode::Lstore3 => "lstore_3",
            Opcode::Fstore0 => "fstore_0",
            Opcode::Fstore1 => "fstore_1",
            Opcode::Fstore2 => "fstore_2",
            Opcode::Fstore3 => "fstore_3",
            Opcode::Dstore0 => "dstore_0",
            Opcode::Dstore1 => "dstore_1",
            Opcode::Dstore2 => "dstore_2",
            Opcode::Dstore3 => "dstore_3",
            Opcode::Astore0 => "astore_0",
            Opcode::Astore1 => "astore_1",
            Opcode::Astore2 => "astore_2",
            Opcode::Astore3 => "astore_3",
            Opcode::Iastore => "iastore",
            Opcode::Lastore => "lastore",
            Opcode::Fastore => "fastore",
            Opcode::Dastore => "dastore",
            Opcode::Aastore => "aastore",
            Opcode::Bastore => "bastore",
            Opcode::Castore => "castore",
            Opcode::Sastore => "sastore",
            Opcode::Pop => "pop",
            Opcode::Pop2 => "pop2",
            Opcode::Dup => "dup",
            Opcode::DupX1 => "dup_x1",
            Opcode::DupX2 => "dup_x2",
            Opcode::Dup2 => "dup2",
            Opcode::Dup2X1 => "dup2_x1",
            Opcode::Dup2X2 => "dup2_x2",
            Opcode::Swap => "swap",
            Opcode::Iadd => "iadd",
            Opcode::Ladd => "ladd",
            Opcode::Fadd => "fadd",
            Opcode::Dadd => "dadd",
            Opcode::Isub => "isub",
            Opcode::Lsub => "lsub",
            Opcode::Fsub => "fsub",
            Opcode::Dsub => "dsub",
            Opcode::Imul => "imul",
            Opcode::Lmul => "lmul",
            Opcode::Fmul => "fmul",
            Opcode::Dmul => "dmul",
            Opcode::Idiv => "idiv",
            Opcode::Ldiv => "ldiv",
            Opcode::Fdiv => "fdiv",
            Opcode::Ddiv => "ddiv",
            Opcode::Irem => "irem",
            Opcode::Lrem => "lrem",
            Opcode::Frem => "frem",
            Opcode::Drem => "drem",
            Opcode::Ineg => "ineg",
            Opcode::Lneg => "lneg",
            Opcode::Fneg => "fneg",
            Opcode::Dneg => "dneg",
            Opcode::Ishl => "ishl",
            Opcode::Lshl => "lshl",
            Opcode::Ishr => "ishr",
            Opcode::Lshr => "lshr",
            Opcode::Iushr => "iushr",
            Opcode::Lushr => "lushr",
            Opcode::Iand => "iand",
            Opcode::Land => "land",
            Opcode::Ior => "ior",
            Opcode::Lor => "lor",
            Opcode::Ixor => "ixor",
            Opcode::Lxor => "lxor",
            Opcode::Iinc => "iinc",
            Opcode::I2l => "i2l",
            Opcode::I2f => "i2f",
            Opcode::I2d => "i2d",
            Opcode::L2i => "l2i",
            Opcode::L2f => "l2f",
            Opcode::L2d => "l2d",
            Opcode::F2i => "f2i",
            Opcode::F2l => "f2l",
            Opcode::F2d => "f2d",
            Opcode::D2i => "d2i",
            Opcode::D2l => "d2l",
            Opcode::D2f => "d2f",
            Opcode::I2b => "i2b",
            Opcode::I2c => "i2c",
            Opcode::I2s => "i2s",
            Opcode::Lcmp => "lcmp",
            Opcode::Fcmpl => "fcmpl",
            Opcode::Fcmpg => "fcmpg",
            Opcode::Dcmpl => "dcmpl",
            Opcode::Dcmpg => "dcmpg",
            Opcode::Ifeq => "ifeq",
            Opcode::Ifne => "ifne",
            Opcode::Iflt => "iflt",
            Opcode::Ifge => "ifge",
            Opcode::Ifgt => "ifgt",
            Opcode::Ifle => "ifle",
            Opcode::IfIcmpeq => "if_icmpeq",
            Opcode::IfIcmpne => "if_icmpne",
            Opcode::IfIcmplt => "if_icmplt",
            Opcode::IfIcmpge => "if_icmpge",
            Opcode::IfIcmpgt => "if_icmpgt",
            Opcode::IfIcmple => "if_icmple",
            Opcode::IfAcmpeq => "if_acmpeq",
            Opcode::IfAcmpne => "if_acmpne",
            Opcode::Goto => "goto",
            Opcode::Jsr => "jsr",
            Opcode::Ret => "ret",
            Opcode::Tableswitch => "tableswitch",
            Opcode::Lookupswitch => "lookupswitch",
            Opcode::Ireturn => "ireturn",
            Opcode::Lreturn => "lreturn",
            Opcode::Freturn => "freturn",
            Opcode::Dreturn => "dreturn",
            Opcode::Areturn => "areturn",
            Opcode::Return => "return",
            Opcode::Getstatic => "getstatic",
            Opcode::Putstatic => "putstatic",
            Opcode::Getfield => "getfield",
            Opcode::Putfield => "putfield",
            Opcode::Invokevirtual => "invokevirtual",
            Opcode::Invokespecial => "invokespecial",
            Opcode::Invokestatic => "invokestatic",
            Opcode::Invokeinterface => "invokeinterface",
            Opcode::Invokedynamic => "invokedynamic",
            Opcode::New => "new",
            Opcode::Newarray => "newarray",
            Opcode::Anewarray => "anewarray",
            Opcode::Arraylength => "arraylength",
            Opcode::Athrow => "athrow",
            Opcode::Checkcast => "checkcast",
            Opcode::Instanceof => "instanceof",
            Opcode::Monitorenter => "monitorenter",
            Opcode::Monitorexit => "monitorexit",
            Opcode::Wide => "wide",
            Opcode::Multianewarray => "multianewarray",
            Opcode::Ifnull => "ifnull",
            Opcode::Ifnonnull => "ifnonnull",
            Opcode::GotoW => "goto_w",
            Opcode::JsrW => "jsr_w",

            Opcode::Breakpoint => "breakpoint",
            Opcode::ReservedFuture => "reserved_future",
            Opcode::Impdep1 => "impdep1",
            Opcode::Impdep2 => "impdep2",
        }
    }
}