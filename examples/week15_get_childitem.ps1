# Week 15 - Get-ChildItem Examples
# Phase 4, Chunk 1: Basic Directory Listing

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
