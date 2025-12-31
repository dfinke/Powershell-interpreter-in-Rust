# Example: Pipeline syntax
Get-Process | Where-Object

# More complex pipeline
Get-Process | Where-Object { $_.CPU -gt 10 } | Select-Object Name, CPU
