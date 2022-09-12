use fdb::Fdb;

#[cfg(target_os = "windows")]
const USAGE: &str = "
Usage:
    fdb.exe <FILE> get <KEY>
    fbd.exe <FILE> delete <KEY>
    fbd.exe <FILE> insert <KEY> <VALUE>
    fbd.exe <FILE> update <KEY> <VALUE>
";

#[cfg(not(target_os = "windows"))]
const USAGE: &str = "
Usage:
    fdb <FILE> get <KEY>
    fbd <FILE> delete <KEY>
    fbd <FILE> insert <KEY> <VALUE>
    fbd <FILE> update <KEY> <VALUE>
";

fn parse_input(args: &Vec<String>) -> (String, String, String, Option<&str>) {
    let file_name = args.get(1).expect(&USAGE).to_owned();
    let action = args.get(2).expect(&USAGE).to_owned();
    let key = args.get(3).expect(&USAGE).to_owned();
    let maybe_value = match args.get(4) {
        Some(v) => Some(v.as_str()),
        None => None,
    };

    (file_name, action, key, maybe_value)
}

fn run(file_name: String, action: String, key: String, maybe_value: Option<&str>) {
    let path = std::path::Path::new(&file_name);
    let mut store = Fdb::open(path).expect("Unable to open file");
    store.load().expect("Unable to load data");

    let raw_key = key.as_bytes();
    match action.as_str() {
        "get" => match store.get(raw_key).unwrap() {
            None => eprintln!("{:?} not found", key),
            Some(value) => println!("{:?}", value),
        },
        "delete" => store.delete(raw_key).unwrap(),
        "insert" => {
            let value = maybe_value.expect(&USAGE).as_ref();
            store.insert(raw_key, value).unwrap()
        }
        "update" => {
            let value = maybe_value.expect(&USAGE).as_ref();
            store.update(raw_key, value).unwrap()
        }
        _ => eprintln!("{}", &USAGE),
    }
}

fn main() {
    let args = std::env::args().collect();
    let (file_name, action, key, maybe_value) = parse_input(&args);

    run(file_name, action, key, maybe_value);
}

#[cfg(test)]
mod cli_tests {
    #[test]
    fn given_wrong_comand_when_it_should_return_usage_msg() {
        assert_eq!(true, true);
    }
}
