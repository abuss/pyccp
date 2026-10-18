from pyccp import Store

def run():
    # x ∧ y = z, x ∨ y = w, x ⊕ y = v over 0/1
    s = Store([[0, 1]] * 5)           # x y z w v
    s.post(("and", (0, 1, 2)))
    s.post(("or", (0, 1, 3)))
    s.post(("xor", (0, 1, 4)))
    s.post(("neq", (0, 1)))           # x != y -> xor is 1
    sol = s.solve()
    assert sol is not None
    assert sol[2] == sol[0] * sol[1]
    assert sol[3] == min(sol[0] + sol[1], 1)
    assert sol[4] == sol[0] ^ sol[1]
    print("boolean_logic:", sol)
    return sol
