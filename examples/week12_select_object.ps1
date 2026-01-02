# Week 12: Select-Object Cmdlet Examples
# Demonstrates property projection and object transformation

# Create some sample process objects for demonstration
$processes = @(
    @{Name="chrome"; CPU=45.2; Id=5678; WorkingSet=512000}
    @{Name="code"; CPU=23.1; Id=9012; WorkingSet=256000}
    @{Name="pwsh"; CPU=5.0; Id=3456; WorkingSet=51200}
    @{Name="explorer"; CPU=15.5; Id=1234; WorkingSet=102400}
    @{Name="System"; CPU=0.0; Id=4; WorkingSet=1024}
)

# Example 1: Select specific properties
# Week 12 Success Criteria: $objects | Select-Object Name, CPU
Write-Output "Example 1: Select Name and CPU properties"
$selected = $processes | Select-Object Name, CPU
Write-Output $selected

# Example 2: Select single property
Write-Output "Example 2: Select only Name property"
$names = $processes | Select-Object Name
Write-Output $names

# Example 3: Select first N objects
# Week 12 Success Criteria: $objects | Select-Object -First 5
Write-Output "Example 3: Select first 3 processes"
$first = $processes | Select-Object -First 3
Write-Output $first

# Example 4: Select last N objects
Write-Output "Example 4: Select last 2 processes"
$last = $processes | Select-Object -Last 2
Write-Output $last

# Example 5: Combine property selection with limiting
Write-Output "Example 5: Select Name and CPU from first 2 processes"
$combined = $processes | Select-Object Name, CPU -First 2
Write-Output $combined

# Example 6: Pass through without parameters
Write-Output "Example 6: Pass through without parameters"
$passthrough = $processes | Select-Object
Write-Output $passthrough

# Example 7: Full pipeline demonstration
Write-Output "Example 7: Full pipeline - Filter and Select"
$result = $processes | Where-Object { $_.CPU -gt 10 } | Select-Object Name, CPU
Write-Output $result

# Week 12 Success Criteria Verification
Write-Output "Week 12 Success Criteria Tests:"

# Test 1: $objects | Select-Object Name, CPU
Write-Output "Test 1: Select Name, CPU"
$test1 = $processes | Select-Object Name, CPU
Write-Output $test1

# Test 2: $objects | Select-Object -First 5
Write-Output "Test 2: Select -First 5"
$test2 = $processes | Select-Object -First 5
Write-Output $test2

Write-Output "Week 12 Examples Complete!"
