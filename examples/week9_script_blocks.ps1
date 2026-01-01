# Week 9: Script Blocks Examples
# Demonstrates script blocks as first-class values

Write-Output "=== Week 9: Script Blocks ==="
Write-Output ""

# Example 1: Simple script block
Write-Output "Example 1: Creating a simple script block"
$add5 = { $_ + 5 }
Write-Output $add5
Write-Output ""

# Example 2: Script block with comparison
Write-Output "Example 2: Script block with comparison"
$filter = { $_ -gt 5 }
Write-Output $filter
Write-Output ""

# Example 3: Script block with string interpolation
Write-Output "Example 3: Script block with string interpolation"
$formatter = { "Value: $_" }
Write-Output $formatter
Write-Output ""

# Example 4: Script block stored in variable
Write-Output "Example 4: Storing script blocks in variables"
$x = 10
$sb = { $x + 5 }
Write-Output $sb
Write-Output ""

# Example 5: Multiple script blocks
Write-Output "Example 5: Multiple script blocks"
$double = { $_ * 2 }
$square = { $_ * $_ }
Write-Output "Double block:"
Write-Output $double
Write-Output "Square block:"
Write-Output $square
Write-Output ""

# Example 6: Week 9 Success Criteria
Write-Output "Example 6: Week 9 Success Criteria"
Write-Output "Creating filter script block: { `$_ -gt 5 }"
$filter = { $_ -gt 5 }
Write-Output "Filter created successfully!"
Write-Output $filter
Write-Output ""

# Example 7: Script block with multiple statements
Write-Output "Example 7: Script block with multiple statements"
$complex = {
    $temp = $_ * 2
    $temp + 10
}
Write-Output $complex
Write-Output ""

# Example 8: Script block with conditional
Write-Output "Example 8: Script block with conditional"
$conditional = {
    if ($_ -gt 10) {
        "Large"
    }
}
Write-Output $conditional
Write-Output ""

Write-Output "=== Week 9 Script Blocks Complete ==="
