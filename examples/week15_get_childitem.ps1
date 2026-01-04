# Week 15 - Get-ChildItem Examples
# Phase 4: File system cmdlets
# Covers: basic listing, -Path, rich properties, filtering, and recursion (-Recurse / -Depth)

# Example 1: List all files and directories in current directory
Write-Output "=== Example 1: Basic Get-ChildItem ==="
Get-ChildItem

# Example 2: Get file names from current directory
Write-Output ""
Write-Output "=== Example 2: Get File Names ==="
Get-ChildItem | Select-Object Name

# Example 3: Filter files with Where-Object
Write-Output ""
Write-Output "=== Example 3: Filter Files ==="
Get-ChildItem | Where-Object { $_.Name -like "*.ps1" } | Select-Object Name

# Example 4: Count files in directory
Write-Output ""
Write-Output "=== Example 4: Count Files ==="
$files = Get-ChildItem
Write-Output "Total items: $($files.Length)"

# Example 5: Process each file with ForEach-Object
Write-Output ""
Write-Output "=== Example 5: Process Each File ==="
Get-ChildItem | ForEach-Object { Write-Output "Found: $($_.Name)" }

# Example 6: Use -Path to list another directory
Write-Output ""
Write-Output "=== Example 6: -Path Parameter ==="
Get-ChildItem -Path "./crates" | Select-Object Name

# Example 7: Show richer file properties
Write-Output ""
Write-Output "=== Example 7: Rich Properties ==="
Get-ChildItem -Path "./crates" | Select-Object Name, Mode, Length, LastWriteTime, Directory

# Example 8: Built-in filtering parameters
Write-Output ""
Write-Output "=== Example 8: -Filter / -Include / -Exclude ==="
Get-ChildItem -Path "./crates" -Recurse -Depth 3 -Filter "*.rs" | Select-Object Name

# Example 9: Recursive listing with depth limiting
Write-Output ""
Write-Output "=== Example 9: -Recurse with -Depth ==="
Get-ChildItem -Path "./crates" -Recurse -Depth 2 | Select-Object Name, Directory
