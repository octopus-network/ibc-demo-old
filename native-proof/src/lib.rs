mod wasm;

use std::cell::RefCell;

use wasm::{HostExternals, MAX_RUNTIME_MEM};

use wasmi::{
    Error, ImportsBuilder, Module, ModuleInstance, ModuleRef, RuntimeValue, LINEAR_MEMORY_PAGE_SIZE,
};

/// create wasm instance
pub fn create_wasm_instance<B: AsRef<[u8]>>(wasm_bin: B) -> Result<ModuleRef, Error> {
    let module = Module::from_buffer(wasm_bin)?;
    let module_resolver = HostExternals::new(
        (MAX_RUNTIME_MEM / LINEAR_MEMORY_PAGE_SIZE.0) as u32,
        RefCell::new(None),
    );

    let imports = ImportsBuilder::new().with_resolver("env", &module_resolver);
    let instance = ModuleInstance::new(&module, &imports)?.assert_no_start();
    Ok(instance)
}


/// pass wasm instance and invoke export function by name and args
pub fn invoke_export(
    instance: &ModuleRef,
    func_name: &str,
    args: &[RuntimeValue],
) -> Result<Option<RuntimeValue>, Error> {
    instance.invoke_export(func_name, args, &mut HostExternals::default())
}


// temp test
#[test]
fn it_should_work_well() {
    use std::fs;

    let wasm = fs::read("../target/debug/wbuild/proof/proof.compact.wasm").expect("file not found");
    let instance = create_wasm_instance(wasm).expect("create wasm error!!!!!!");
    let args = [RuntimeValue::I32(1), RuntimeValue::I32(2)];
    let res = invoke_export(&instance, "add", &args).unwrap().unwrap();
    assert_eq!(res, RuntimeValue::I32(3));

    let res = invoke_export(&instance, "check_read_proof", &[]).unwrap().unwrap();
    assert_eq!(res, RuntimeValue::I32(1));
}
