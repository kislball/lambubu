import random

def build_expr(op, a, b):
    if op == "SUCC" or op == "PRED":
        return f"({op} {a})"
    else:
        return f"({op} {a} {b})"

def generate(depth = 4):
    op = random.choice([ "ADD", "MULT", "SUCC", "PRED" ])
    if depth == 0:
        return build_expr(op, random.randint(10, 50), random.randint(10, 50))
    else:
        return build_expr(op, generate(depth - 1), generate(depth - 1))

print(generate())
