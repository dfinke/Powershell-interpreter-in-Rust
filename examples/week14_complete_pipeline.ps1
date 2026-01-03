# Week 14: Complete Object Pipeline
# Demonstrates Get-Process cmdlet and the full pipeline

Write-Output "=== Week 14: Complete Object Pipeline ==="
Write-Output ""

# Week 14 Success Criteria: Full pipeline demonstration
Write-Output "1. Week 14 Success Criteria: Complete Pipeline"
Write-Output "Get-Process | Where-Object { $_.CPU -gt 10 } | Select-Object Name, CPU"
Get-Process | Where-Object { $_.CPU -gt 10 } | Select-Object Name, CPU
Write-Output ""

# Example 2: List all processes
Write-Output "2. List all processes"
Get-Process
Write-Output ""

# Example 3: Filter by name
Write-Output "3. Filter processes by name (chrome)"
Get-Process -Name chrome
Write-Output ""

# Example 4: Select specific properties
Write-Output "4. Select Id and Name properties"
Get-Process | Select-Object Id, Name
Write-Output ""

# Example 5: Filter and select
Write-Output "5. Filter by CPU and select Name only"
Get-Process | Where-Object { $_.CPU -gt 20 } | Select-Object Name
Write-Output ""

# Example 6: Get first 3 processes
Write-Output "6. Get first 3 processes"
Get-Process | Select-Object -First 3
Write-Output ""

# Example 7: Complex pipeline with transformation
Write-Output "7. Filter, select, and transform process data"
Get-Process | Where-Object { $_.CPU -gt 5 } | Select-Object Name, CPU -First 2
Write-Output ""

# Example 8: Extract property values
Write-Output "8. Get list of process names only"
Get-Process | ForEach-Object -MemberName Name
Write-Output ""

# Example 9: Filter and extract single property
Write-Output "9. Get names of high CPU processes"
Get-Process | Where-Object { $_.CPU -gt 15 } | ForEach-Object -MemberName Name
Write-Output ""

# Example 10: Complete end-to-end pipeline
Write-Output "10. Full pipeline: filter, select, limit"
Get-Process | Where-Object { $_.CPU -gt 10 } | Select-Object Name, CPU, WorkingSet -First 2
Write-Output ""

Write-Output "=== Week 14 Complete - Object Pipeline Milestone Reached! ==="
