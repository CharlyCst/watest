use serde_yaml::{from_str, Number, Value};
use std::path::PathBuf;

pub struct Module {
    pub funs: Vec<Fun>,
    pub path: PathBuf,
}

pub struct Fun {
    pub name: String,
    pub args: Args,
    pub out: Type,
    pub test: Option<Test>,
}

pub struct Test {
    pub inputs: Vec<Vec<Number>>,
    pub outputs: Option<Vec<Vec<Number>>>,
}

pub enum Type {
    I32,
    I64,
    F32,
    F64,
}

type Args = Vec<Type>;

pub fn parse(yaml: &str) -> Result<Module, String> {
    let yaml = match from_str::<Value>(&yaml) {
        Ok(yaml) => yaml,
        Err(err) => {
            return Err(err.to_string());
        }
    };
    parse_root(yaml)
}

fn parse_root(val: Value) -> Result<Module, String> {
    match val {
        Value::Mapping(map) => {
            let mut wasm_file = None;
            let mut funs = Vec::new();
            for (key, val) in &map {
                if let Value::String(s) = key {
                    match s as &str {
                        "file" => wasm_file = Some(parse_file(val)?),
                        "funs" => funs.extend(parse_funs(val)?),
                        _ => return Err(format!("Unknown key in root object: '{}'.", s)),
                    }
                } else {
                    return Err(String::from("Root keys must be string litterals."));
                }
            }
            if let Some(path) = wasm_file {
                Ok(Module { path, funs })
            } else {
                Err(String::from("Missing wasm file path."))
            }
        }
        _ => Err(String::from("The root declaration must be a mapping")),
    }
}

fn parse_file(val: &Value) -> Result<PathBuf, String> {
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

fn parse_funs(val: &Value) -> Result<Vec<Fun>, String> {
    match val {
        Value::Mapping(map) => {
            let mut funs = Vec::new();
            for (key, val) in map {
                if let Value::String(fun_name) = key {
                    let mut fun = parse_fun(val)?;
                    fun.name = fun_name.clone();
                    funs.push(fun);
                } else {
                    return Err(String::from("Function names must be string literrals."));
                }
            }
            Ok(funs)
        }
        _ => Err(String::from("Functions ('funs') must be a mapping.")),
    }
}

fn parse_fun(val: &Value) -> Result<Fun, String> {
    match val {
        Value::Mapping(map) => {
            let mut args = None;
            let mut out = None;
            let mut test = None;
            for (key, val) in map {
                if let Value::String(attribute) = key {
                    match attribute as &str {
                        "args" => {
                            args = Some(parse_args(val)?);
                        }
                        "out" => {
                            out = Some(parse_out(val)?);
                        }
                        "test" => {
                            test = Some(parse_test(val)?);
                        }
                        _ => return Err(format!("Unknown function attribute '{}'.", attribute)),
                    }
                } else {
                    return Err(String::from(
                        "Function attributes must be string litterals.",
                    ));
                }
            }
            if let Some(args) = args {
                if let Some(out) = out {
                    Ok(Fun {
                        name: String::from(""),
                        args,
                        out,
                        test,
                    })
                } else {
                    Err(String::from("Missing return type in function."))
                }
            } else {
                Err(String::from("Missing arguments type in function."))
            }
        }
        _ => Err(String::from("Functions (`fun`) must be mappings.")),
    }
}

fn parse_args(val: &Value) -> Result<Args, String> {
    match val {
        Value::Sequence(seq) => {
            let mut args = Vec::with_capacity(seq.len());
            for arg in seq {
                if let Value::String(s) = arg {
                    args.push(type_from_str(&s)?);
                } else {
                    return Err(String::from("Argument type must be a string litteral."));
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

fn parse_out(val: &Value) -> Result<Type, String> {
    match val {
        Value::String(s) => type_from_str(&s),
        _ => Err(String::from("Expect a string litteral for `out`.")),
    }
}

fn parse_test(val: &Value) -> Result<Test, String> {
    match val {
        Value::Mapping(map) => {
            let mut inputs = None;
            let mut outputs = None;
            for (key, val) in map {
                if let Value::String(s) = key {
                    match s as &str {
                        "with" => inputs = Some(parse_test_with(val)?),
                        "expect" => outputs = Some(parse_test_expect(val)?),
                        _ => return Err(format!("Unknown key in func object: '{}'.", s)),
                    }
                } else {
                    return Err(String::from("Root keys must be string litterals."));
                }
            }
            if let Some(inputs) = inputs {
                Ok(Test { inputs, outputs })
            } else {
                Err(String::from(
                    "A `test` must include input values with `with` declaration.",
                ))
            }
        }
        _ => Err(String::from(
            "A function `test` attribute must be a mapping.",
        )),
    }
}

fn parse_test_with(val: &Value) -> Result<Vec<Vec<Number>>, String> {
    match val {
        Value::Sequence(seq) => {
            let mut test_inputs = Vec::new();
            for inputs in seq {
                let values = match inputs {
                    Value::Sequence(seq) => {
                        let mut values = Vec::with_capacity(seq.len());
                        for val in seq {
                            if let Value::Number(n) = val {
                                values.push(n.clone());
                            } else {
                                return Err(String::from("Test values must be numbers."));
                            };
                        }
                        values
                    },
                    Value::Number(n) => vec![n.clone()],
                    _ => return Err(String::from("Test input values (`with` attribute) must be either numbers or sequences of numbers."))
                };
                test_inputs.push(values)
            }
            Ok(test_inputs)
        }
        _ => Err(String::from(
            "The `with` attribute of test must be a sequence.",
        )),
    }
}

fn parse_test_expect(val: &Value) -> Result<Vec<Vec<Number>>, String> {
    match val {
        Value::Sequence(seq) => {
            let mut test_outputs = Vec::new();
            for inputs in seq {
                let values = match inputs {
                    Value::Sequence(seq) => {
                        let mut values = Vec::with_capacity(seq.len());
                        for val in seq {
                            if let Value::Number(n) = val {
                                values.push(n.clone());
                            } else {
                                return Err(String::from("Test values must be numbers."));
                            };
                        }
                        values
                    },
                    Value::Number(n) => vec![n.clone()],
                    _ => return Err(String::from("Test output values (`expect` attribute) must be either numbers or sequences of numbers."))
                };
                test_outputs.push(values)
            }
            Ok(test_outputs)
        }
        _ => Err(String::from(
            "The `expect` attribute of test must be a sequence.",
        )),
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
