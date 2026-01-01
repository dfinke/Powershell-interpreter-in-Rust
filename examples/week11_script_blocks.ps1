# Week 11: Script Blocks in Cmdlets
# Demonstrates Where-Object and ForEach-Object with script blocks

Write-Output "=== Week 11: Script Blocks in Cmdlets ==="
Write-Output ""

# Week 11 Success Criteria: Array filtering with Where-Object
Write-Output "1. Week 11 Success Criteria: Filter array with Where-Object"
@(1,2,3,4,5) | Where-Object { $_ -gt 2 }
Write-Output ""

# ForEach-Object with transformation
Write-Output "2. Transform values with ForEach-Object"
@(1,2,3,4,5) | ForEach-Object { $_ * 2 }
Write-Output ""

# Complex filtering
Write-Output "3. Filter numbers greater than 5"
@(1,2,3,4,5,6,7,8,9,10) | Where-Object { $_ -gt 5 }
Write-Output ""

# Chained operations
Write-Output "4. Chain Where-Object and ForEach-Object"
@(1,2,3,4,5,6,7,8,9,10) | Where-Object { $_ -gt 5 } | ForEach-Object { $_ * 2 }
Write-Output ""

# Get-Process with filtering
Write-Output "5. Filter processes by CPU usage"
Get-Process | Where-Object { $_.CPU -gt 10 }
Write-Output ""

# Select specific properties
Write-Output "6. Select specific properties"
Get-Process | Select-Object -Property "Name" | ForEach-Object { $_.Name }
Write-Output ""

Write-Output "=== Week 11 Complete ==="
