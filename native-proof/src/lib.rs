mod wasm;

use wasm::HostExternals;
use wasmi::{Error, Externals, ImportsBuilder, Module, ModuleInstance, ModuleRef, RuntimeValue, ModuleImportResolver};

pub fn create_wasm_instance<B: AsRef<[u8]>>(wasm_bin: B) -> Result<ModuleRef, Error> {
    let module = Module::from_buffer(wasm_bin)?;
    let imports = ImportsBuilder::new()
            .with_resolver("env", resolver: &'a dyn ModuleImportResolver);
    let instance = ModuleInstance::new(&module, &imports)?.assert_no_start();
    Ok(instance)
}

pub fn invoke_export(instance: &ModuleRef, func_name: &str, args: &[RuntimeValue]) -> Result<Option<RuntimeValue>, Error> {
    instance.invoke_export(func_name, args, &mut HostExternals {})
}

#[test]
fn it_should_work_well() {
    use std::fs;
    
    let wasm = fs::read("../target/debug/wbuild/proof/proof.compact.wasm").expect("file not found");
    let instance = create_wasm_instance(wasm).expect("create wasm error!!!!!!");
    let args = [
        RuntimeValue::I32(1),
        RuntimeValue::I32(2),
    ];
    let res = invoke_export(&instance, "add", &args).expect("invoke error").expect("none");
    assert_eq!(res, RuntimeValue::I32(3))
}