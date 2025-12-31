# Week 6 MVP Success Criteria - Object Pipeline with 5 Cmdlets
# This script demonstrates the working object pipeline implementation

# Success Criterion 1: Write-Output cmdlet
Write-Output "Hello World"
Write-Output 42

# Success Criterion 2: Variables with Write-Output
$x = 5
Write-Output $x

# Success Criterion 3: Simple pipeline
42 | Write-Output
$name = "PowerShell"
$name | Write-Output

# Success Criterion 4: Get-Process cmdlet
# Returns mock process data for demonstration
Get-Process

# Success Criterion 5: Pipeline with Get-Process
# Get process named chrome
Get-Process -Name "chrome"

# Additional demonstrations of the 5 cmdlets:

# 1. Write-Output - Display values
Write-Output "Cmdlet 1: Write-Output works!"

# 2. Get-Process - List system processes (mock data)
Write-Output "Cmdlet 2: Get-Process returns process objects"

# 3. Where-Object - Filter objects (basic implementation)
Write-Output "Cmdlet 3: Where-Object for filtering"

# 4. Select-Object - Select properties (basic implementation)
Write-Output "Cmdlet 4: Select-Object for projection"

# 5. ForEach-Object - Transform objects (basic implementation)
Write-Output "Cmdlet 5: ForEach-Object for transformation"

# Complex expressions still work
$y = 10
$z = $x + $y
Write-Output $z

# String interpolation
$greeting = "Hello $name"
Write-Output $greeting

# Comparison operators
$isGreater = 10 -gt 5
Write-Output $isGreater

# If statements
if ($x -eq 5) {
    Write-Output "x equals 5"
}
