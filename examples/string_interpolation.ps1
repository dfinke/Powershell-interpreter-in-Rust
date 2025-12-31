# Example: String interpolation in PowerShell

# Simple interpolation
$name = "Alice"
"Hello $name"

# Multiple variables
$first = "John"
$last = "Doe"
"Full name: $first $last"

# Interpolation with expressions
$x = 10
$y = 20
"Sum: $x + $y"

# Escaped dollar sign (literal $)
"Price: \$100"

# Single quotes do not interpolate
'Hello $name'  # Outputs: Hello $name
