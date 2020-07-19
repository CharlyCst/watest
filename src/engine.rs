use serde_yaml::Number as YamlNumber;
use wasmtime::{Engine, Instance, Module, Store, Val};

use crate::parser::{Fun, Module as ModuleSpec, Type};
use crate::error::ErrorHandler;

pub fn run_test(module_spec: ModuleSpec) -> ErrorHandler {
    let mut handler = ErrorHandler::new();
    let engine = Engine::default();
    let module = match Module::from_file(&engine, &module_spec.path) {
        Ok(module) => module,
        Err(_) => {
            eprintln!("Unable to open wasm module: path '{}' does not exist or is not a valid module.", &module_spec.path.to_str().unwrap_or("PATH_ERROR"));
            handler.silent_report();
            return handler;
        }
    };
    let store = Store::new(&engine);
    let instance = Instance::new(&store, &module, &[]).unwrap();

    for fun in &module_spec.funs {
        test_fun(fun, &instance, &mut handler);
    }

    handler
}

fn test_fun(fun: &Fun, instance: &Instance, handler: &mut ErrorHandler) {
    let callable = instance.get_func(&fun.name);
    let callable = if let Some(callable) = callable {
        callable
    } else {
        return handler.report(fun.name.clone(), format!("No function named '{}'.", fun.name));
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
            return handler.report(fun.name.clone(), format!("The number of expected output must match the number of inputs. Got {}, expected {}.", outputs.len(), inputs.len()));
        }
        for (input, output) in inputs.iter().zip(outputs.iter()) {
            let input = match prepare_values(&fun.args, input) {
                Ok(input) => input,
                Err(err) => {
                    handler.report(fun.name.clone(), err);
                    continue;
                }
            };
            let result = match callable.call(&input) {
                Ok(result) => {
                    result
                }
                Err(_) => {
                    handler.report(fun.name.clone(), String::from("Function trapped."));
                    continue;
                }
            };
            if let Err(err) = test_equality(output, result.as_ref()) {
                handler.report(fun.name.clone(), err)
            }
        }
    }
}

fn prepare_values(types: &Vec<Type>, values: &Vec<YamlNumber>) -> Result<Vec<Val>, String> {
    let mut prepared_values = Vec::with_capacity(values.len());

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
                        return Err(format!("Expected {}, got {}", y, f64::from_bits(*x)));
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
