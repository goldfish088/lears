use std::env;
use std::process;

mod scanner {
}

// fn print_type<T>(_: &T) {
//     println!("type is: {}", std::any::type_name::<T>());
// }

fn main() {
    let num_args = env::args().len();
    if num_args > 2 {
        let fullpath = env::args().next().unwrap();

        // NOTE: only UNIX compatible...
        println!("Usage: ./{} [script]", match fullpath.rfind('/') {
            Some(i) => String::from(&fullpath[i+1..]),
            _ => fullpath
        });

        process::exit(64);
    }

    todo!();

}
