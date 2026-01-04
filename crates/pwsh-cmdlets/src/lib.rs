mod foreach_object;
mod get_childitem;
mod get_content;
mod get_process;
mod select_object;
mod where_object;
/// PowerShell built-in cmdlets
mod write_output;

// Re-export cmdlets
pub use foreach_object::ForEachObjectCmdlet;
pub use get_childitem::GetChildItemCmdlet;
pub use get_content::GetContentCmdlet;
pub use get_process::GetProcessCmdlet;
pub use select_object::SelectObjectCmdlet;
pub use where_object::WhereObjectCmdlet;
pub use write_output::WriteOutputCmdlet;

/// Register all built-in cmdlets
pub fn register_all(registry: &mut pwsh_runtime::CmdletRegistry) {
    registry.register(Box::new(WriteOutputCmdlet));
    registry.register(Box::new(WhereObjectCmdlet));
    registry.register(Box::new(SelectObjectCmdlet));
    registry.register(Box::new(ForEachObjectCmdlet));
    registry.register(Box::new(GetProcessCmdlet));
    registry.register(Box::new(GetChildItemCmdlet));
    registry.register(Box::new(GetContentCmdlet));
}
