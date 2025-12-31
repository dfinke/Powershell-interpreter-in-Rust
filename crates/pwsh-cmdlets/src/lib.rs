/// PowerShell built-in cmdlets
mod write_output;
mod where_object;
mod select_object;
mod foreach_object;
mod get_process;

// Re-export cmdlets
pub use write_output::WriteOutputCmdlet;
pub use where_object::WhereObjectCmdlet;
pub use select_object::SelectObjectCmdlet;
pub use foreach_object::ForEachObjectCmdlet;
pub use get_process::GetProcessCmdlet;

/// Register all built-in cmdlets
pub fn register_all(registry: &mut pwsh_runtime::CmdletRegistry) {
    registry.register(Box::new(WriteOutputCmdlet));
    registry.register(Box::new(WhereObjectCmdlet));
    registry.register(Box::new(SelectObjectCmdlet));
    registry.register(Box::new(ForEachObjectCmdlet));
    registry.register(Box::new(GetProcessCmdlet));
}

