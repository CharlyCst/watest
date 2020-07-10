use wasmtime::{Engine, Instance, Module, Store};

use crate::parser::{FunTest, Test};

pub fn run_test(test: Test) {
    let engine = Engine::default();
    let module = Module::from_file(&engine, test.path).unwrap();
    let store = Store::new(&engine);
    let instance = Instance::new(&store, &module, &[]).unwrap();

    let add = instance
        .get_func("_start")
        .expect("`_start` is not exported");
    let add = add
        .get2::<i32, i32, i32>()
        .expect("function has a bad signature");
    println!("{:?}", add(40, 2));

    for fun in &test.funs {
        test_fun(fun, &instance)
    }
}

fn test_fun(fun_test: &FunTest, instance: &Instance) {
    let fun = instance.get_func(&fun_test.name);
    let fun = if let Some(fun) = fun {
        fun
    } else {
        println!("No function named '{}'.", fun_test.name);
        return;
    };
}
