use serde_yaml::Number as YamlNumber;
use wasmtime::{Engine, Instance, Module, Store, Val};

use crate::parser::{Fun, Module as ModuleSpec, Type};

pub fn run_test(module_spec: ModuleSpec) {
    let engine = Engine::default();
    let module = Module::from_file(&engine, module_spec.path).unwrap();
    let store = Store::new(&engine);
    let instance = Instance::new(&store, &module, &[]).unwrap();

    for fun in &module_spec.funs {
        match test_fun(fun, &instance) {
            Ok(_) => (),
            Err(err) => println!("{}", err),
        }
    }
}

fn test_fun(fun: &Fun, instance: &Instance) -> Result<(), String> {
    let callable = instance.get_func(&fun.name);
    let callable = if let Some(callable) = callable {
        callable
    } else {
        return Err(format!("No function named '{}'.", fun.name));
    };

    if let Some(test) = &fun.test {
        let inputs = &test.inputs;
        let outputs = if let Some(outputs) = &test.outputs {
            outputs.clone()
        } else {
            let mut outputs = Vec::with_capacity(inputs.len());
            for _ in 0..inputs.len() {
                outputs.push(vec![]);
            }
            outputs
        };

        if inputs.len() != outputs.len() {
            return Err(format!("The number of expected output must match the number of inputs. Got {}, expected {}.", outputs.len(), inputs.len()));
        }
        for (input, output) in inputs.iter().zip(outputs.iter()) {
            let input = match prepare_values(&fun.args, input) {
                Ok(input) => input,
                Err(err) => {
                    println!("{}", err);
                    continue;
                }
            };
            let result = match callable.call(&input) {
                Ok(result) => {
                    // println!("Success: {:?}", result);
                    result
                }
                Err(_) => {
                    println!("Function trapped.");
                    continue;
                }
            };
            if let Err(err) = test_equality(output, result.as_ref()) {
                println!("{}", err);
            }
        }
    }
    Ok(())
}

fn prepare_values(types: &Vec<Type>, values: &Vec<YamlNumber>) -> Result<Vec<Val>, String> {
    let mut prepared_values = Vec::with_capacity(values.len());
    // Err(String::from("YAML error: could not cast numbers into the correct types."));

    if types.len() != values.len() {
        return Err(format!(
            "Expected {} values for input but got '{}'.",
            types.len(),
            values.len()
        ));
    }
    for (val, t) in values.iter().zip(types.iter()) {
        let prepared_value = match t {
            Type::I32 => match val.as_i64() {
                Some(n) => Val::I32(n as i32),
                None => return Err(format!("Failed to interpret '{}' as i32.", val.to_string())),
            },
            Type::I64 => match val.as_i64() {
                Some(n) => Val::I64(n),
                None => return Err(format!("Failed to interpret '{}' ad i64.", val.to_string())),
            },
            Type::F32 => match val.as_f64() {
                Some(x) => Val::F32((x as f32).to_bits()),
                None => return Err(format!("Failed to interpret '{}' as f32.", val.to_string())),
            },
            Type::F64 => match val.as_f64() {
                Some(x) => Val::F64(x.to_bits()),
                None => return Err(format!("Failed to interpret '{}' as f64.", val.to_string())),
            },
        };
        prepared_values.push(prepared_value);
    }
    Ok(prepared_values)
}

fn test_equality(target: &Vec<YamlNumber>, result: &[Val]) -> Result<(), String> {
    if target.len() != result.len() {
        return Err(format!("Expected {} values, got {}.", target.len(), result.len()));
    }
    for (target, result) in target.iter().zip(result.iter()) {
        match result {
            Val::I32(n) => {
                if let Some(m) = target.as_i64() {
                    if *n != m as i32 {
                        return Err(format!("Expected {}, got {}", m, n));
                    }
                } else {
                    return Err(String::from("Unexpected return type"));
                }
            }
            Val::I64(n) => {
                if let Some(m) = target.as_i64() {
                    if *n != m {
                        return Err(format!("Expected {}, got {}", m, n));
                    }
                } else {
                    return Err(String::from("Unexpected return type"));
                }
            }
            Val::F32(x) => {
                if let Some(y) = target.as_f64() {
                    if *x != (y as f32).to_bits() {
                        return Err(format!("Expected {}, got {}", y as f32, f32::from_bits(*x)));
                    }
                } else {
                    return Err(String::from("Unexpected return type"));
                }
            }
            Val::F64(x) => {
                if let Some(y) = target.as_f64() {
                    if *x != y.to_bits() {
                        return Err(format!("Expected {}, got {}", y, x));
                    }
                } else {
                    return Err(String::from("Unexpected return type"));
                }
            }
            _ => return Err(String::from("Unexpected return type")),
        }
    }
    Ok(())
}
