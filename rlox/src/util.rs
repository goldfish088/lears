#[allow(dead_code)]
pub fn print_type<T>(_: &T) {
    println!("&type = {}", std::any::type_name::<&T>());
}
