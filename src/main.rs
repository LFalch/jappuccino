use std::env::args;

use jappuccino::rt;

fn main() {
    let mut args = args().skip(1);
    let class = args.next().unwrap();
    let args: Box<[_]> = args.map(String::into_boxed_str).collect();
    let mut rt = rt::Runtime::new();
    rt.run(&class, args).unwrap();
}
