use wasmi::{
    Error, Externals, FuncInstance, FuncRef, ImportsBuilder, Module, ModuleImportResolver,
    ModuleInstance, ModuleRef, RuntimeArgs, RuntimeValue, Signature, Trap, ValueType,
};

mod env {
    use super::*;

    pub const ADD_FUNC_INDEX: usize = 0;
    pub const CHECK_READ_PROOF: usize = 1;

    pub fn check_read_proof(params: *const u8, len: usize) -> Result<Option<RuntimeValue>, Trap> {
        Ok(Some(RuntimeValue::I32(0)))
    }

    pub fn add(args: RuntimeArgs) -> Result<Option<RuntimeValue>, Trap> {
        let a: u32 = args.nth_checked(0)?;
        let b: u32 = args.nth_checked(1)?;
        let result = a + b;
        Ok(Some(RuntimeValue::I32(result as i32)))
    }
}

pub struct HostExternals;

impl HostExternals {
    fn check_signature(&self, index: usize, signature: &Signature) -> bool {
        let (params, ret_ty): (&[ValueType], Option<ValueType>) = match index {
            env::ADD_FUNC_INDEX => (&[ValueType::I32, ValueType::I32], Some(ValueType::I32)),
            _ => return false,
        };
        signature.params() == params && signature.return_type() == ret_ty
    }
}

impl Externals for HostExternals {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            env::ADD_FUNC_INDEX => env::add(args),

            env::CHECK_READ_PROOF => env::check_read_proof(&0, 0),

            _ => panic!("Unimplemented function at {}", index),
        }
    }
}


impl ModuleImportResolver for HostExternals {
    fn resolve_func(&self, field_name: &str, signature: &Signature) -> Result<FuncRef, Error> {
        // set func position
        let index = match field_name {
            "ext_add" => env::ADD_FUNC_INDEX,
            "ext_check_read_proof" => env::CHECK_READ_PROOF,

            _ => {
                return Err(Error::Instantiation(format!(
                    "Export {} not found",
                    field_name
                )));
            }
        };

        // check func signature
        if !self.check_signature(index, signature) {
            return Err(Error::Instantiation(format!(
                "Export {} has a bad signature",
                field_name
            )));
        }

        // alloc position for func
        Ok(FuncInstance::alloc_host(
            Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
            index,
        ))
    }
}
