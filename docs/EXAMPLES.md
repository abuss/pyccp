# Examples

| Example | Description | Status |
|---|---|---|
| agent_counter | Agents in a ring passing messages via ports (ping/pong lifecycle demo) | done |
| agent_counter_random | Same ring, but each agent forwards to a **random** ring neighbor | done |
| agent_inbox_monitoring | Inspecting agent inbox queue depth with `InboxLen()`/`InboxCap()` | v2: no inbox introspection in v1 |
| boolean_logic | Boolean logic constraints: AND, OR, XOR, implication, equivalence | done |
| cardinality | Count constraint — how many variables equal a given value | done |
| circuit | Hamiltonian cycle constraint (TSP variants via permutation cycles) | done (v2.5.2: permutation-based, cycle validation available) |
| cyclic | Modulo scheduling — shift type determined by `day mod 3` | done |
| diophantine | Linear Diophantine equation: find all pairs x + y = 10 | done |
| element | Array indexing constraint `y = array[index]`, bidirectional propagation | done |
| graph_coloring | Graph coloring: every pair of K4 nodes differs, needs 4 colours | done |
| knapsack | 0/1 knapsack: maximize value under weight limit | done |
| linear | Linear program (GLOP port): maximize x+y under linear constraints | v2: LP backend |
| magic_square | 3x3 magic square: every line sums to 15 | done |
| modulo_filter | Enumerate all x in 1–100 divisible by 3 but not 4 or 7 | done |
| names_demo | Names API: unique 32-char hex agent identifiers (`NewName()`/`IsName()`) | v2: no names API in v1 |
| nqueens | N-Queens CSP: AllDifferent + diagonal constraints, multi-solution search | done |
| parallel_sudoku | Sudoku solver using AC-4 propagation | v2: specialized propagator (AC-4) |
| producer_consumer | Producer and consumer agents, value committed on one var read from the other | done |
| scaling | Multiplication constraint: find width × height = 12 | done |
| scheduling | Job assignment: minimize makespan across machines | done |
| search | Search with asymmetric domains, x < y | done |
| send_more_money | SEND + MORE = MONEY cryptarithmetic | done |
| simple | Minimal CSP: x < y with same domains | done |
| sudoku | 9x9 Sudoku solver using AllDiff constraints on rows, columns, and boxes | done (v2.5.2: generic solver, AC-4 optimization available in v2.6) |
| strategy_choice | Demonstrate strategy choice and search phases | done |
| tsp | Travelling salesmen via branch-and-bound optimization | done |

## Notes

**v2.5.2 New Examples:**
- **sudoku.py** — Generic 9x9 Sudoku solver using AllDiff constraints. Demonstrates row/column/box constraints. Runs correctly without AC-4 propagator; AC-4 optimization deferred to v2.6+.
- **circuit.py** — Hamiltonian circuit/permutation solver using AllDiff. Includes validation utilities to check for single-cycle tours. Full circuit propagator (subtour elimination) deferred to v2.6+.

**v2+ Deferred Examples:**
- **parallel_sudoku** — Requires AC-4 specialized propagator (optimization only)
- **linear** — Requires LP backend (major feature)
- **agent_inbox_monitoring** — Requires Inbox API
- **names_demo** — Requires Names/symbols API
