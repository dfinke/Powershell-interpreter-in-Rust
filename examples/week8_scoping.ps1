# Week 8: Advanced Scoping Examples
# Demonstrates scope qualifiers: $global:, $local:, $script:

# ===================================
# Example 1: Global Scope Basics
# ===================================
$global:appName = "PowerShell Interpreter"
Write-Output "Application: $global:appName"

# ===================================
# Example 2: Global Counter
# ===================================
$global:counter = 0

function IncrementCounter {
    $global:counter = $global:counter + 1
    Write-Output "Counter incremented to: $global:counter"
}

IncrementCounter
IncrementCounter
IncrementCounter
Write-Output "Final counter value: $global:counter"

# ===================================
# Example 3: Local vs Global Scope
# ===================================
$x = 100

function ShowScopes {
    $local:x = 200
    Write-Output "Local x: $local:x"
    Write-Output "Global x: $global:x"
}

ShowScopes
Write-Output "Outside function, x is: $x"

# ===================================
# Example 4: Week 8 Success Criteria
# ===================================
# From ROADMAP: Week 8 success criteria
$global:x = 5

function Test {
    $local:y = 10
    $x + $y
}

$result = Test
Write-Output "Week 8 Success: 5 + 10 = $result"

# ===================================
# Example 5: Script Scope
# ===================================
# Script scope currently behaves like global scope
$script:version = "1.0.0"
Write-Output "Script version: $script:version"

# ===================================
# Example 6: Nested Functions with Scope
# ===================================
$global:total = 0

function AddToTotal($amount) {
    $global:total = $global:total + $amount
    Write-Output "Added $amount, total is now: $global:total"
}

function ProcessItems {
    $local:item1 = 10
    $local:item2 = 20
    $local:item3 = 30
    
    AddToTotal $item1
    AddToTotal $item2
    AddToTotal $item3
}

ProcessItems
Write-Output "Final total: $global:total"

# ===================================
# Example 7: Scope Qualifier Case Insensitivity
# ===================================
$GLOBAL:config = "Production"
Write-Output "Config (GLOBAL): $global:config"

$Global:debug = true
Write-Output "Debug (Global): $GLOBAL:debug"

# ===================================
# Example 8: Mixed Scope Operations in Functions
# ===================================
$global:a = 1

function MixedScopes {
    $local:b = 2
    $script:c = 3
    
    $sum = $global:a + $local:b + $script:c
    Write-Output "Sum of different scopes: $sum"
}

MixedScopes

# ===================================
# Example 9: Shadowing with Local Scope
# ===================================
$name = "Global Name"

function ChangeName {
    # This creates a local variable, doesn't change global
    $local:name = "Local Name"
    Write-Output "Inside function: $name"
}

ChangeName
Write-Output "Outside function: $name"

# ===================================
# Example 10: Explicit Global Modification
# ===================================
$global:shared = "Original"

function ModifyShared {
    # Explicitly modify the global variable
    $global:shared = "Modified"
}

Write-Output "Before: $global:shared"
ModifyShared
Write-Output "After: $global:shared"

Write-Output ""
Write-Output "Week 8 scoping examples completed successfully!"
