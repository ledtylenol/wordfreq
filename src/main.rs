use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{Read, stdin},
};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let num = args.len();
    println!("{args:?}, {num}");
    let filename = args
        .get(1)
        .expect("primul agument trebuie sa fie numele fileului de citit");
    let mut data = String::new();

    let mut f = match File::open(filename) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("eroare de filesystem: {e}");
            return;
        }
    };

    let collect_to_hashmap = |mut acc: HashMap<_, _>, elem| {
        acc.entry(elem).and_modify(|e| *e += 1).or_insert(1);
        acc
    };
    f.read_to_string(&mut data).expect("could not read file");
    let data = data
        .split_whitespace()
        .fold(HashMap::new(), collect_to_hashmap);
    let mut data = data.into_iter().collect::<Vec<_>>();
    data.sort_by(|a, b| b.1.cmp(&a.1));
    for args in args.windows(2) {
        let (Some(arg1), Some(arg2)) = (args.get(0), args.get(1)) else {
            continue;
        };
        if let (true, Ok(num)) = (arg1 == "--top", arg2.parse::<usize>()) {
            for (i, (word, count)) in data.iter().take(num).enumerate() {
                println!("top {} word: {word} with {count} appearances", i + 1);
            }
        }
    }
}
