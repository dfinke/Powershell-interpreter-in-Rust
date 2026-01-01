# Week 10: Object System - Hashtable Creation and Property Access
# This script demonstrates the new object system features implemented in Week 10

# ==================
# 1. Empty Hashtable
# ==================
Write-Output "1. Empty hashtable:"
$empty = @{}
Write-Output $empty

# ==================
# 2. Simple Hashtable
# ==================
Write-Output ""
Write-Output "2. Simple hashtable with string and number:"
$person = @{Name="John"; Age=30}
Write-Output $person

# ==================
# 3. Property Access
# ==================
Write-Output ""
Write-Output "3. Accessing hashtable properties:"
$person = @{Name="John"; Age=30}
Write-Output "Name:"
Write-Output $person.Name
Write-Output "Age:"
Write-Output $person.Age

# ==================
# 4. Complex Hashtable
# ==================
Write-Output ""
Write-Output "4. More complex hashtable:"
$employee = @{FirstName="Alice"; LastName="Smith"; Department="Engineering"; Salary=75000}
Write-Output "Employee Details:"
Write-Output $employee
Write-Output "First Name:"
Write-Output $employee.FirstName
Write-Output "Department:"
Write-Output $employee.Department

# ==================
# 5. Week 10 Success Criteria (from ROADMAP.md)
# ==================
Write-Output ""
Write-Output "5. Week 10 Success Criteria:"
$obj = @{Name="John"; Age=30}
Write-Output "obj.Name:"
Write-Output $obj.Name
Write-Output "obj.Age:"
Write-Output $obj.Age

# ==================
# 6. Nested Values
# ==================
Write-Output ""
Write-Output "6. Hashtable with expressions:"
$data = @{X=5; Y=10; Sum=15}
Write-Output "Data:"
Write-Output $data
Write-Output "X value:"
Write-Output $data.X
Write-Output "Y value:"
Write-Output $data.Y

# ==================
# 7. Multiple Hashtables
# ==================
Write-Output ""
Write-Output "7. Multiple independent hashtables:"
$car = @{Make="Toyota"; Model="Camry"; Year=2020}
$house = @{Address="123 Main St"; Bedrooms=3; Price=350000}
Write-Output "Car make:"
Write-Output $car.Make
Write-Output "House address:"
Write-Output $house.Address

Write-Output ""
Write-Output "Week 10 demonstrations complete!"
