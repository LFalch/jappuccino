use crate::rt::RuntimeCtx;

pub fn obj_init(ctx: &mut RuntimeCtx) {
    let _this = ctx.pop();
}
pub fn obj_equals(ctx: &mut RuntimeCtx) {
    let this = ctx.pop();
    let obj = ctx.pop();
    ctx.push(this == obj);
}
pub fn obj_get_class(ctx: &mut RuntimeCtx) {
    let this = ctx.pop();
    let class_instance = ctx.get_class_object(this);
    ctx.push(class_instance);
}
pub fn obj_hash_code(ctx: &mut RuntimeCtx) {
    let this = ctx.pop();
    ctx.push(this);
}
pub fn obj_to_string(ctx: &mut RuntimeCtx) {
    let this = ctx.pop();
    let hash_code = this.into_u32();
    let class_name = ctx.get_class_name(this);

    let string = ctx.new_string_obj(format!("{class_name}@{hash_code:x}"));
    ctx.push(string);
}
pub fn printstream_println_str(ctx: &mut RuntimeCtx) {
    let sref = ctx.pop();
    let this = ctx.pop();
    let n = ctx.read_u8_ref(this.offset(0)).unwrap();

    let sref = ctx.read_string_object(sref).unwrap();

    match n {
        0 => println!("{sref}"),
        _ => unreachable!(),
    }
}
