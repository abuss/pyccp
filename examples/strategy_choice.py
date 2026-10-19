import asyncio
import pyccp

def run():
    """
    Multiple competing solution strategies using choice().
    
    Demonstrates: pyccp.choice(*agents) races multiple strategies
    and returns the result of the first one to complete.
    
    Use case: when you have several heuristics or search orders,
    try them in parallel and use whichever succeeds first.
    
    Note: Both strategies here solve the same problem, so they're equally
    fast. In practice, you'd use different orderings, heuristics, or
    even completely different constraint formulations.
    """
    async def main():
        # Simple constraint satisfaction: x + y == 5, x != y
        domains = [[0, 1, 2, 3, 4, 5], [0, 1, 2, 3, 4, 5]]
        
        async def strategy_1():
            """Strategy 1: Post sum constraint first, then inequality."""
            s = pyccp.Store(domains)
            s.post(("linear_eq", ([(0, 1), (1, 1)], 5)))  # x + y == 5
            s.post(("neq", (0, 1)))  # x != y
            return s.solve()
        
        async def strategy_2():
            """Strategy 2: Post inequality first, then sum constraint."""
            s = pyccp.Store(domains)
            s.post(("neq", (0, 1)))  # x != y
            s.post(("linear_eq", ([(0, 1), (1, 1)], 5)))  # x + y == 5
            return s.solve()
        
        # Race both strategies: first to find solution wins
        solution = await pyccp.choice(strategy_1, strategy_2)
        
        assert solution is not None
        assert len(solution) == 2
        assert solution[0] + solution[1] == 5, f"Sum constraint violated: {solution}"
        assert solution[0] != solution[1], f"Inequality constraint violated: {solution}"
        
        print(f"strategy_choice: solution {solution} found via first-to-complete strategy")
        return solution
    
    return asyncio.run(main())


if __name__ == "__main__":
    run()
