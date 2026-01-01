# Week 7: Function Definitions and Calls
# This script demonstrates all the function features implemented in Week 7

# Simple function with no parameters
function SayHello() {
    "Hello, World!"
}

# Function with parameters
function Add($a, $b) {
    $a + $b
}

# Function with default parameters
function Greet($name = "Guest") {
    "Hello, $name!"
}

# Function with explicit return
function Double($x) {
    return $x * 2
}

# Function with conditional logic
function Max($a, $b) {
    if ($a -gt $b) {
        return $a
    } else {
        return $b
    }
}

# Function with multiple statements and implicit return
function Calculate($x) {
    $doubled = $x * 2
    $added = $doubled + 10
    $added
}

# Nested function calls
function Triple($x) {
    $x * 3
}

function Sextuple($x) {
    Triple (Double $x)
}

# Test the functions
Write-Output "=== Week 7 Function Examples ==="

Write-Output "SayHello:"
SayHello

Write-Output "Add 5 10:"
Add 5 10

Write-Output "Greet (default):"
Greet

Write-Output "Greet Alice:"
Greet "Alice"

Write-Output "Double 21:"
Double 21

Write-Output "Max 10 5:"
Max 10 5

Write-Output "Calculate 5:"
Calculate 5

Write-Output "Sextuple 7:"
Sextuple 7
