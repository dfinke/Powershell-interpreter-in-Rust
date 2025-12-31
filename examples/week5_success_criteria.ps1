# Week 5 Success Criteria Test Script
# Tests the Runtime & Evaluator implementation

# Basic variable assignment and arithmetic
$x = 5
$y = 10
$z = $x + $y
# $z should be 15

# Complex expressions
$result = ($x + $y) * 2
# $result should be 30

# String interpolation
$name = "PowerShell"
$greeting = "Hello $name"
# $greeting should be "Hello PowerShell"

# Comparison operators
$isGreater = 10 -gt 5
# $isGreater should be true

$isEqual = 5 -eq 5
# $isEqual should be true

# If statements
if ($x -gt 3) {
    $message = "X is greater than 3"
}
# $message should be "X is greater than 3"

# If-else statements
if ($y -lt 5) {
    $status = "Small"
} else {
    $status = "Large"
}
# $status should be "Large"

# Nested expressions
$a = 2
$b = 3
$c = 4
$complex = $a * $b + $c
# $complex should be 10 (2 * 3 + 4)
