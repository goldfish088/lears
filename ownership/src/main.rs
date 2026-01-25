// use std::str;

// fn take_ownership(s: String) -> String {
//     println!("I took ownership of s!");
//     s
// }

// fn get_type<T>(_: &T) -> String {
//     format!("{:?}", std::any::type_name::<T>())
// }

fn find_first_word(s: &String) -> String {
    let mut start: usize = 0;
    let mut end: usize;

    {
        let mut bytes = s.bytes();

        let mut curr = bytes.next();
        while curr.is_some() && curr == Some(b' ') {
            start += 1;
            curr = bytes.next();
        }

        end = start;
        while curr.is_some() && curr != Some(b' ') {
            end += 1;
            curr = bytes.next();
        }
    }

    String::from(&s[start..end])
}

fn fuzz_with_strings(sentences: &Vec<&str>) {
    for sentence in sentences {
        let s = String::from(*sentence);
        println!("the first word of '{}' is '{}'", s, find_first_word(&s)); 
    }
}

fn main() {
    fuzz_with_strings(&["hi", "byte", "    another long    sequence of words", "word"].to_vec());
}
