# Week 16 - Get-Content Examples
# Phase 4: File system cmdlets
# Chunk 1: Basic File Reading

Write-Output "=== Example 1: Read a file ==="
Get-Content -Path "./README.md" | Selwect-Object Length

Write-Output ""
Write-Output "=== Example 2: Read a file (positional path) ==="
Get-Content "./README.md" | Select-Object Length

Write-Output ""
Write-Output "=== Example 3: Specify encoding ==="
# Supported values currently include: UTF8, ASCII, Unicode (UTF-16LE), BigEndianUnicode (UTF-16BE)
Get-Content -Path "./README.md" -Encoding 'UTF8' | Select-Object Length
