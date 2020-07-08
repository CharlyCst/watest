use serde_yaml::{from_str, Value};
use std::path::PathBuf;

pub enum Type {
    I32,
    I64,
    F32,
    F64,
}

type Args = Vec<Type>;

pub fn parse(yaml: &str) {
    let yaml = match from_str::<Value>(&yaml) {
        Ok(yaml) => yaml,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };
    println!("{:#?}", yaml);
    match root(yaml) {
        Ok(()) => println!("Parsed!"),
        Err(err) => println!("{}", err),
    }
}

fn root(val: Value) -> Result<(), String> {
    match val {
        Value::Mapping(map) => {
            let mut wasm_file: Option<PathBuf> = None;
            for (key, val) in &map {
                if let Value::String(s) = key {
                    println!("{}", s);
                    match s as &str {
                        "file" => wasm_file = Some(file(val)?),
                        "funs" => funs(val)?,
                        _ => return Err(format!("Unknown key in root object: '{}'.", s)),
                    }
                } else {
                    return Err(String::from("Root keys must be string litterals."));
                }
            }
            Ok(())
        }
        _ => Err(String::from("The root declaration must be a mapping")),
    }
}

fn file(val: &Value) -> Result<PathBuf, String> {
    match val {
        Value::String(file_name) => {
            let path = PathBuf::from(file_name);
            if path.is_file() {
                Ok(path)
            } else {
                Err(format!("File path '{}' does not exist.", file_name))
            }
        }
        _ => Err(String::from("File path must be a string litteral.")),
    }
}

fn funs(val: &Value) -> Result<(), String> {
    match val {
        Value::Mapping(map) => {
            for (key, val) in map {
                if let Value::String(fun_name) = key {
                    fun(val)?;
                } else {
                    return Err(String::from("Function names must be string literrals."));
                }
            }
            Ok(())
        }
        _ => Err(String::from("Functions ('funs') must be a mapping.")),
    }
}

fn fun(val: &Value) -> Result<(), String> {
    match val {
        Value::Mapping(map) => {
            for (key, val) in map {
                if let Value::String(attribute) = key {
                    match attribute as &str {
                        "args" => {
                            args(val)?;
                        }
                        "out" => {
                            out(val)?;
                        }
                        "test" => (),
                        _ => return Err(format!("Unknown function attribute '{}'.", attribute)),
                    }
                } else {
                    return Err(String::from(
                        "Function attributes must be string litterals.",
                    ));
                }
            }
            Ok(())
        }
        _ => Err(String::from("Functions (`fun`) must be mappings.")),
    }
}

fn args(val: &Value) -> Result<Args, String> {
    match val {
        Value::Sequence(seq) => {
            let mut args = Vec::with_capacity(seq.len());
            for arg in seq {
                if let Value::String(s) = arg {
                    args.push(type_from_str(&s)?);
                } else {
                    let err = String::from("Argument type must be a string litteral.");
                    return Err(err);
                }
            }
            Ok(args)
        }
        Value::String(s) => Ok(vec![type_from_str(&s)?]),
        _ => Err(String::from(
            "Expect either a list or a string literal for `args`.",
        )),
    }
}

fn out(val: &Value) -> Result<Type, String> {
    match val {
        Value::String(s) => type_from_str(&s),
        _ => Err(String::from("Expect a string litteral for `out`.")),
    }
}

fn type_from_str(s: &str) -> Result<Type, String> {
    match s {
        "i32" => Ok(Type::I32),
        "i64" => Ok(Type::I64),
        "f32" => Ok(Type::F32),
        "f64" => Ok(Type::F64),
        _ => Err(String::from("Valid types are 'i32', 'i64', 'f32', 'f64'.")),
    }
}
