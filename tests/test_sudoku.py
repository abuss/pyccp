"""
Tests for Sudoku CSP solver (v2.5.2 example).

Tests validate that Sudoku puzzles are correctly solved using AllDiff constraints.
"""

import pytest
import pyccp


class TestSudokuBasic:
    """Basic Sudoku solving tests."""

    def test_solve_empty_sudoku_all_solutions(self):
        """Empty 9x9 Sudoku has 6.67 billion solutions; just verify solver runs."""
        # For performance, we limit to 100 solutions (demo only)
        board = [
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0],
        ]
        solutions = solve_sudoku(board, limit=100)
        # Should find at least one solution (actually billions, but limit to 100)
        assert len(solutions) > 0
        # First solution should be a valid completed board
        assert len(solutions[0]) == 81

    def test_solve_easy_sudoku_single_solution(self):
        """Easy Sudoku puzzle with unique solution."""
        # This is an easy puzzle with only a few blanks
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
        solutions = solve_sudoku(board, limit=10)
        # Easy puzzle should have unique or few solutions
        assert len(solutions) >= 1
        # Verify first solution is valid
        sol = solutions[0]
        assert len(sol) == 81
        # Each cell in solution should be 1-9
        assert all(1 <= v <= 9 for v in sol)

    def test_sudoku_respects_initial_values(self):
        """Solver respects initial puzzle values; doesn't override them."""
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
        solutions = solve_sudoku(board, limit=10)
        assert len(solutions) > 0
        sol = solutions[0]
        
        # Verify fixed cells from board are in solution
        # Cell (0,0) = 5, so solution[0] should equal 5
        assert sol[0] == 5
        assert sol[1] == 3
        # Cell (1,0) = 6
        assert sol[9] == 6
        # Cell (1,3) = 1
        assert sol[12] == 1

    def test_sudoku_all_different_rows(self):
        """All values in each row are different (1-9, no duplicates)."""
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
        solutions = solve_sudoku(board, limit=10)
        assert len(solutions) > 0
        sol = solutions[0]
        
        # Check each row has all 1-9 (no duplicates)
        for row_idx in range(9):
            row = sol[row_idx * 9:(row_idx + 1) * 9]
            assert len(set(row)) == 9, f"Row {row_idx} has duplicates: {row}"
            assert set(row) == set(range(1, 10)), f"Row {row_idx} has wrong values: {row}"

    def test_sudoku_all_different_columns(self):
        """All values in each column are different (1-9, no duplicates)."""
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
        solutions = solve_sudoku(board, limit=10)
        assert len(solutions) > 0
        sol = solutions[0]
        
        # Check each column has all 1-9
        for col_idx in range(9):
            col = [sol[row_idx * 9 + col_idx] for row_idx in range(9)]
            assert len(set(col)) == 9, f"Column {col_idx} has duplicates: {col}"
            assert set(col) == set(range(1, 10)), f"Column {col_idx} has wrong values: {col}"

    def test_sudoku_all_different_boxes(self):
        """All values in each 3x3 box are different (1-9, no duplicates)."""
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
        solutions = solve_sudoku(board, limit=10)
        assert len(solutions) > 0
        sol = solutions[0]
        
        # Check each 3x3 box has all 1-9
        for box_row in range(3):
            for box_col in range(3):
                box = []
                for i in range(3):
                    for j in range(3):
                        row_idx = box_row * 3 + i
                        col_idx = box_col * 3 + j
                        box.append(sol[row_idx * 9 + col_idx])
                assert len(set(box)) == 9, f"Box ({box_row},{box_col}) has duplicates: {box}"
                assert set(box) == set(range(1, 10)), f"Box ({box_row},{box_col}) has wrong values: {box}"


def solve_sudoku(board, limit=1):
    """
    Solve a 9x9 Sudoku puzzle using AllDiff constraints.
    
    Args:
        board: 9x9 list of lists; 0 = empty cell, 1-9 = initial values
        limit: Max number of solutions to find
    
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
    
    # Create store
    store = pyccp.Store(domains)
    
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
