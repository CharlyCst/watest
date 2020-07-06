use wasmtime::{ Engine, Store, Module, Instance };

fn main() {
    println!("Hello, world!");

    let engine = Engine::default();
    let module = Module::from_file(&engine, "wasm/add.wasm").unwrap();
    let store = Store::new(&engine);
    let instance = Instance::new(&store, &module, &[]).unwrap();

    let add = instance.get_func("_start").expect("`_start` is not exported");
    let add = add.get2::<i32, i32, i32>().expect("function has a bad signature");
    println!("{:?}", add(40, 2));
}
