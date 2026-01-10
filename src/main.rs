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

    println!("dati numele fisierului: ");

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
    for (k, v) in data {
        println!("cuvantul {k} este prezent de {v} ori");
    }
}
