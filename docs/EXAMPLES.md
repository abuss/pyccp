# Examples

| Example | Description |
|---|---|
| agent_counter | Agents in a ring passing messages via ports (ping/pong lifecycle demo) |
| agent_counter_random | Same ring, but each agent forwards to a **random** ring neighbor |
| agent_inbox_monitoring | Inspecting agent inbox queue depth with `InboxLen()`/`InboxCap()` |
| boolean | Boolean logic constraints: AND, OR, XOR, implication, equivalence |
| cardinality | Count constraint — how many variables equal a given value |
| circuit | Hamiltonian cycle constraint (TSP variants via permutation cycles) |
| cyclic | Modulo scheduling — shift type determined by `day mod 3` |
| diophantine | Linear Diophantine equation: find all pairs x + y = 10 |
| element | Array indexing constraint `y = array[index]`, bidirectional propagation |
| knapsack | 0/1 knapsack: maximize value under weight limit |
| linear | Linear program (GLOP port): maximize x+y under linear constraints |
| modulo_constraints | Enumerate all x in 1–100 divisible by 3 but not 4 or 7 |
| names_demo | Names API: unique 32-char hex agent identifiers (`NewName()`/`IsName()`) |
| nqueens | N-Queens CSP: AllDifferent + diagonal constraints, multi-solution search |
| parallel_sudoku | Sudoku solver using AC-4 propagation |
| scaling | Multiplication constraint: find width × height = 12 |
| scheduling | Job assignment: minimize makespan across machines |
| search | Search with asymmetric domains, x < y |
| sendmoremoney | SEND + MORE = MONEY cryptarithmetic (partially implemented) |
| simple | Minimal CSP: x < y with same domains |
| tsp | Travelling salesmen via branch-and-bound optimization |
