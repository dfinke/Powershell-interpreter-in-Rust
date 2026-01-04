# Week 16 - Get-Content Examples
# Phase 4: File system cmdlets
# Chunk 1: Basic File Reading

Write-Output "=== Example 1: Read a file ==="
Get-Content -Path "./README.md" | Select-Object Length

Write-Output ""
Write-Output "=== Example 2: Read a file (positional path) ==="
Get-Content "./README.md" | Select-Object Length
