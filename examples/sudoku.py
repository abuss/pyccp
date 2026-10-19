"""
Sudoku CSP Solver (v2.5.2)

Solves 9x9 Sudoku puzzles using AllDiff constraints on rows, columns, and 3x3 boxes.
No AC-4 propagator needed - correctness over speed.
"""

from pyccp import Store


def solve_sudoku(board, limit=1):
    """
    Solve a 9x9 Sudoku puzzle using AllDiff constraints.
    
    Args:
        board: 9x9 list of lists; 0 = empty cell, 1-9 = initial values
        limit: Max number of solutions to find (default: 1)
    
    Returns:
        List of solutions, each as a flat list of 81 values (1-9)
    """
    # Create domain for each cell: [1-9] for empty, [fixed_value] for given
    domains = []
    for i in range(9):
        for j in range(9):
            if board[i][j] == 0:
                domains.append(list(range(1, 10)))
            else:
                domains.append([board[i][j]])
    
    # Create constraint store
    store = Store(domains)
    
    # Add AllDiff constraint for each row
    for row_idx in range(9):
        cells = [row_idx * 9 + col_idx for col_idx in range(9)]
        store.post(("all_diff", (cells,)))
    
    # Add AllDiff constraint for each column
    for col_idx in range(9):
        cells = [row_idx * 9 + col_idx for row_idx in range(9)]
        store.post(("all_diff", (cells,)))
    
    # Add AllDiff constraint for each 3x3 box
    for box_row in range(3):
        for box_col in range(3):
            cells = []
            for i in range(3):
                for j in range(3):
                    row_idx = box_row * 3 + i
                    col_idx = box_col * 3 + j
                    cells.append(row_idx * 9 + col_idx)
            store.post(("all_diff", (cells,)))
    
    # Solve and return solutions
    solutions = store.solve_all(limit=limit)
    return solutions


def format_board(flat_solution):
    """Convert flat solution (81 values) to 2D board (9x9)."""
    board = []
    for i in range(9):
        row = flat_solution[i * 9:(i + 1) * 9]
        board.append(row)
    return board


def print_board(board):
    """Pretty-print a 9x9 Sudoku board."""
    for i, row in enumerate(board):
        if i % 3 == 0 and i != 0:
            print("-" * 25)
        row_str = ""
        for j, val in enumerate(row):
            if j % 3 == 0 and j != 0:
                row_str += "| "
            row_str += str(val) + " "
        print(row_str)


def run():
    """Example: solve a 9x9 Sudoku puzzle."""
    # Easy example: from Wikipedia
    board = [
        [5, 3, 0, 0, 7, 0, 0, 0, 0],
        [6, 0, 0, 1, 9, 5, 0, 0, 0],
        [0, 9, 8, 0, 0, 0, 0, 6, 0],
        [8, 0, 0, 0, 6, 0, 0, 0, 3],
        [4, 0, 0, 8, 0, 3, 0, 0, 1],
        [7, 0, 0, 0, 2, 0, 0, 0, 6],
        [0, 6, 0, 0, 0, 0, 2, 8, 0],
        [0, 0, 0, 4, 1, 9, 0, 0, 5],
        [0, 0, 0, 0, 8, 0, 0, 7, 9],
    ]
    
    print("Sudoku puzzle (0 = empty):")
    print_board(board)
    print("\nSolving...")
    
    solutions = solve_sudoku(board)
    
    if solutions:
        print(f"\n✅ Found solution:\n")
        solved = format_board(solutions[0])
        print_board(solved)
        return solutions[0]
    else:
        print("❌ No solution found (puzzle may be invalid)")
        return None


if __name__ == "__main__":
    run()
