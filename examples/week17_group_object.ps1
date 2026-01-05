# Week 17 - Group-Object examples

# Group simple values
@(1, 2, 2, 3, 3, 3) | Group-Object | Write-Output

# Group files by extension
# Get-ChildItem now includes an Extension property.
Get-ChildItem -Path ./examples | Group-Object Extension | Select-Object Name, Count | Write-Output

# Hashtable output
$groups = @("a", "b", "b", "c") | Group-Object -AsHashTable true
$groups | Write-Output

# NoElement output (omit Group arrays)
@(1, 1, 2, 2, 2) | Group-Object -NoElement true | Write-Output
