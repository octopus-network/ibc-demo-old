use std::cell::RefCell;

use wasmi::{
    memory_units, Error, Externals, FuncInstance, FuncRef, MemoryDescriptor, MemoryInstance,
    MemoryRef, ModuleImportResolver, RuntimeArgs, RuntimeValue, Signature, Trap, ValueType,
};

mod env {
    use super::*;

    pub const ADD_FUNC_INDEX: usize = 0;
    pub const CHECK_READ_PROOF: usize = 1;

    pub fn check_read_proof() -> Result<Option<RuntimeValue>, Trap> {
        Ok(Some(RuntimeValue::I32(0)))
    }

    pub fn add(args: RuntimeArgs) -> Result<Option<RuntimeValue>, Trap> {
        let a: u32 = args.nth_checked(0)?;
        let b: u32 = args.nth_checked(1)?;
        let result = a + b;
        Ok(Some(RuntimeValue::I32(result as i32)))
    }
}

// maximum memory in bytes
pub const MAX_RUNTIME_MEM: usize = 1024 * 1024 * 1024; // 1 GiB
pub const MAX_CODE_MEM: usize = 16 * 1024 * 1024; // 16 MiB

/// HostExternals for index host functions and resolve importer
#[derive(Debug, Default)]
pub struct HostExternals {
    max_memory: u32, // in pages.
    memory: RefCell<Option<MemoryRef>>,
}

impl HostExternals {
    /// Create a host external interface
    pub fn new(max_memory: u32, memory: RefCell<Option<MemoryRef>>) -> Self {
        Self { max_memory, memory }
    }

    fn check_signature(&self, index: usize, signature: &Signature) -> bool {
        let (params, ret_ty): (&[ValueType], Option<ValueType>) = match index {
            env::ADD_FUNC_INDEX => (&[ValueType::I32, ValueType::I32], Some(ValueType::I32)),
            env::CHECK_READ_PROOF => (&[], Some(ValueType::I32)),

            _ => return false,
        };
        signature.params() == params && signature.return_type() == ret_ty
    }
}

// for index functions
impl Externals for HostExternals {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            env::ADD_FUNC_INDEX => env::add(args),

            env::CHECK_READ_PROOF => env::check_read_proof(),

            _ => panic!("Unimplemented function at {}", index),
        }
    }
}

// resolve name to host functions
impl ModuleImportResolver for HostExternals {
    fn resolve_func(&self, field_name: &str, signature: &Signature) -> Result<FuncRef, Error> {
        match field_name {
            "ext_add" => {
                let index = env::ADD_FUNC_INDEX;
                 Ok(FuncInstance::alloc_host(
                    Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
                    index,
                ))
            },
            "ext_check_read_proof" => {
                let index = env::CHECK_READ_PROOF;
                Ok(FuncInstance::alloc_host(
                    Signature::new(&[][..], Some(ValueType::I32)),
                    index,
                ))
            }

            _ => {
                return Err(Error::Instantiation(format!(
                    "Export {} not found",
                    field_name
                )));
            }
        }
    }

    fn resolve_memory(
        &self,
        field_name: &str,
        descriptor: &MemoryDescriptor,
    ) -> Result<MemoryRef, Error> {
        if field_name == "memory" {
            let effective_max = descriptor.maximum().unwrap_or(self.max_memory);
            if descriptor.initial() > self.max_memory || effective_max > self.max_memory {
                Err(Error::Instantiation(
                    "Module requested too much memory".to_owned(),
                ))
            } else {
                let mem = MemoryInstance::alloc(
                    memory_units::Pages(descriptor.initial() as usize),
                    descriptor
                        .maximum()
                        .map(|x| memory_units::Pages(x as usize)),
                )?;
                *self.memory.borrow_mut() = Some(mem.clone());
                Ok(mem)
            }
        } else {
            Err(Error::Instantiation(
                "Memory imported under unknown name".to_owned(),
            ))
        }
    }
}
