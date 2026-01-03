# Week 13: ForEach-Object Cmdlet
# Demonstrates object transformation and pipeline mapping

Write-Output "=== Week 13: ForEach-Object Cmdlet ==="
Write-Output ""

# Week 13 Success Criteria: Transform values with ForEach-Object
Write-Output "1. Week 13 Success Criteria: 1..10 | ForEach-Object { $_ * 2 }"
Write-Output "(Note: Range operator not yet implemented, using array literal)"
@(1,2,3,4,5,6,7,8,9,10) | ForEach-Object { $_ * 2 }
Write-Output ""

# Example 2: Transform with different operation
Write-Output "2. Square each number"
@(1,2,3,4,5) | ForEach-Object { $_ * $_ }
Write-Output ""

# Example 3: Access object properties with -MemberName
Write-Output "3. Extract Name property using -MemberName"
$obj1 = @{Name="Alice"; Age=30}
$obj2 = @{Name="Bob"; Age=25}
@($obj1, $obj2) | ForEach-Object -MemberName Name
Write-Output ""

# Example 4: String manipulation
Write-Output "4. Add prefix to strings"
@("apple", "banana", "cherry") | ForEach-Object { "Fruit: $_" }
Write-Output ""

# Example 5: Chained pipeline operations
Write-Output "5. Chain Where-Object and ForEach-Object"
@(1,2,3,4,5,6,7,8,9,10) | Where-Object { $_ -gt 5 } | ForEach-Object { $_ * 3 }
Write-Output ""

# Example 6: Process data
Write-Output "6. Transform process objects"
Get-Process | ForEach-Object -MemberName Name
Write-Output ""

# Example 7: Combine operations on filtered data
Write-Output "7. Filter and transform"
Get-Process | Where-Object { $_.CPU -gt 10 } | ForEach-Object -MemberName CPU
Write-Output ""

Write-Output "=== Week 13 Complete ==="
