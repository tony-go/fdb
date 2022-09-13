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

fn parse_input(args: &Vec<String>) -> (&str, &str, &str, Option<&str>) {
    let file_name = args.get(1).expect(&USAGE).as_ref();
    let action = args.get(2).expect(&USAGE).as_ref();
    let key = args.get(3).expect(&USAGE).as_ref();
    let maybe_value = match args.get(4) {
        Some(v) => Some(v.as_str()),
        None => None,
    };

    (file_name, action, key, maybe_value)
}

pub fn run(
    file_name: &str,
    action: &str,
    key: &str,
    maybe_value: Option<&str>,
) -> Result<String, String> {
    let path = std::path::Path::new(&file_name);
    let mut store = Fdb::open(path).expect("Unable to open file");
    store.load().expect("Unable to load data");

    let raw_key = key.as_bytes();
    match action {
        "get" => match store.get(raw_key).unwrap() {
            None => Err(format!("{:?} not found", key)),
            Some(value) => Ok(std::str::from_utf8(&value).unwrap().to_string()),
        },
        "delete" => {
            store.delete(raw_key).unwrap();
            Ok(key.to_string())
        }
        "insert" => {
            let value = maybe_value.expect(&USAGE).as_ref();
            store.insert(raw_key, value).unwrap();
            Ok(key.to_string())
        }
        "update" => {
            let value = maybe_value.expect(&USAGE).as_ref();
            store.update(raw_key, value).unwrap();
            Ok(key.to_string())
        }
        _ => Err(format!("{}", &USAGE)),
    }
}

fn main() {
    let args = std::env::args().collect();
    let (file_name, action, key, maybe_value) = parse_input(&args);

    let res = run(file_name, action, key, maybe_value);

    match res {
        Ok(result) => println!("{}", result),
        Err(error) => eprintln!("{}", error),
    }
}

#[cfg(test)]
mod cli_tests {
    use super::run;
    use std::fs::remove_file;

    const TEST_FILE: &str = "fdb-test.db";

    fn clean() {
        remove_file(TEST_FILE).expect("Impossible to clean test suit.");
    }

    #[test]
    fn insert_value_and_retrieve_it() {
        let key = "foo";
        let value = "bar";

        run(TEST_FILE, "insert", key, Some(value)).expect("Impossible to insert foo: bar");
        let res = run(TEST_FILE, "get", key, None).expect("Impossible to get foo.");

        assert_eq!(res, value);
        clean();
    }

    #[test]
    fn retrieve_not_inserted_value() {
        let error = run(TEST_FILE, "get", "undefined_key", None);

        assert!(error.is_err());
        clean();
    }

    #[test]
    fn cannot_retrieve_deleted_value() {
        let key = "foo";
        let value = "bar";

        run(TEST_FILE, "insert", key, Some(value)).expect("Impossible to insert foo: bar");
        run(TEST_FILE, "delete", key, None).expect("Impossible to insert foo: bar");

        let error = run(TEST_FILE, "get", key, None);

        assert!(error.is_err());
        clean();
    }
}
