import fileinput

last_depth = 0
increases = 0
for line in fileinput.input():
    if line.strip() == '':
        break
    new_depth = int(line.strip())
    if new_depth > last_depth:
        increases += 1
    last_depth = new_depth
print(increases)
