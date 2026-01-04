mod foreach_object;
mod get_childitem;
mod get_content;
mod get_process;
mod select_object;
mod set_content;
mod where_object;
/// PowerShell built-in cmdlets
mod write_output;

// Re-export cmdlets
pub use foreach_object::ForEachObjectCmdlet;
pub use get_childitem::GetChildItemCmdlet;
pub use get_content::GetContentCmdlet;
pub use get_process::GetProcessCmdlet;
pub use select_object::SelectObjectCmdlet;
pub use set_content::SetContentCmdlet;
pub use where_object::WhereObjectCmdlet;
pub use write_output::WriteOutputCmdlet;

/// Return the names of all built-in cmdlets registered by `register_all`.
///
/// This is intended for UI/REPL features like autocomplete.
pub fn cmdlet_names() -> Vec<String> {
    vec![
        "Write-Output".to_string(),
        "Where-Object".to_string(),
        "Select-Object".to_string(),
        "ForEach-Object".to_string(),
        "Get-Process".to_string(),
        "Get-ChildItem".to_string(),
        "Get-Content".to_string(),
        "Set-Content".to_string(),
    ]
}

/// Register all built-in cmdlets
pub fn register_all(registry: &mut pwsh_runtime::CmdletRegistry) {
    registry.register(Box::new(WriteOutputCmdlet));
    registry.register(Box::new(WhereObjectCmdlet));
    registry.register(Box::new(SelectObjectCmdlet));
    registry.register(Box::new(ForEachObjectCmdlet));
    registry.register(Box::new(GetProcessCmdlet));
    registry.register(Box::new(GetChildItemCmdlet));
    registry.register(Box::new(GetContentCmdlet));
    registry.register(Box::new(SetContentCmdlet));
}
