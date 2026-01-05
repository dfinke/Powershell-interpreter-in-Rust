# Week 17 - Sort-Object examples

# Sort simple values
@(3, 1, 4, 1, 5, 9) | Sort-Object | Write-Output

# Descending (switch params are passed as explicit booleans in this interpreter)
@(3, 1, 4, 1, 5, 9) | Sort-Object -Descending true | Write-Output

# Sort objects by a property
$procs = @(
    @{Name = "chrome"; CPU = 45.2; WorkingSet = 512000 }
    @{Name = "pwsh"; CPU = 5.0; WorkingSet = 51200 }
    @{Name = "code"; CPU = 23.1; WorkingSet = 256000 }
)

$procs | Sort-Object CPU | Select-Object Name, CPU | Write-Output

# Sort files by length
Get-ChildItem | Sort-Object Length -Descending true | Select-Object Name, Length | Write-Output
