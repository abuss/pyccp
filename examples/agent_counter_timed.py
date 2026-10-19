import asyncio
import pyccp
import time

def run():
    async def main():
        """
        Multiple independent rings with agents passing incremented values.
        
        Architecture:
        - 4 concurrent independent rings, each with 4 agents
        - Each ring: agent i reads value, increments, sends to agent (i+1)%4
        - When a ring completes (all 4 agents commit): create fresh ring and restart
        - Runs for 1 second, cycling through multiple complete ring passes
        
        Example progression:
          Ring 0: value 0→1→2→3 [complete, 4 commits] → create new ring, repeat
          Ring 1: value 0→1→2→3 [complete, 4 commits] → create new ring, repeat
          Ring 2: value 0→1→2→3 [complete, 4 commits] → create new ring, repeat
          Ring 3: value 0→1→2→3 [complete, 4 commits] → create new ring, repeat
        
        Messages counted: successful commit() operations only (true message passing)
        """
        num_rings = 4
        agents_per_ring = 4
        max_value = 1000  # Domain large enough
        
        commit_count = [0]  # Track successful commits across all rings
        final_value = [None]
        dispatcher_tasks = []
        
        def create_ring():
            """Create a new independent ring store and ports."""
            store = pyccp.Store([list(range(max_value))] * agents_per_ring)
            ports = [store.var(f"p{i}", i) for i in range(agents_per_ring)]
            return {"store": store, "ports": ports}
        
        # Initialize all 4 rings
        rings = [create_ring() for _ in range(num_rings)]
        
        async def agent(ring_idx, agent_idx, ports):
            """Agent reads from its port, increments, commits to next port."""
            v = await ports[agent_idx].read()
            
            # Attempt to commit to next agent's port
            next_idx = (agent_idx + 1) % agents_per_ring
            target = ports[next_idx]
            
            if not target.is_fixed():
                await target.commit(v + 1)
                commit_count[0] += 1
                final_value[0] = v + 1
        
        async def run_ring_instance(ring_idx):
            """Run one complete cycle of a single ring."""
            ring = rings[ring_idx]
            
            try:
                await pyccp.join(
                    *[lambda ai=i, p=ring["ports"]: agent(ring_idx, ai, p) for i in range(agents_per_ring)]
                )
            except Exception:
                pass
        
        # Start dispatcher for each ring
        for ring_idx in range(num_rings):
            d = asyncio.create_task(pyccp.dispatcher(rings[ring_idx]["store"]))
            dispatcher_tasks.append((ring_idx, d))
        
        # Initialize all rings with starting value 0
        for ring_idx in range(num_rings):
            await rings[ring_idx]["ports"][0].commit(0)
        
        # Run rings for 1 second
        wall_start = time.time()
        start_time = asyncio.get_event_loop().time()
        cycle_count = [0]
        
        while asyncio.get_event_loop().time() - start_time < 1.0:
            # Run one pass around each ring
            for ring_idx in range(num_rings):
                await run_ring_instance(ring_idx)
                
                # Check if this ring completed a full cycle (all ports fixed)
                all_fixed = all(rings[ring_idx]["ports"][i].is_fixed() for i in range(agents_per_ring))
                
                if all_fixed:
                    # Ring cycle complete, create fresh ring for next iteration
                    cycle_count[0] += 1
                    rings[ring_idx] = create_ring()
                    
                    # Restart dispatcher for new ring
                    old_dispatcher = next(d for idx, d in dispatcher_tasks if idx == ring_idx)
                    old_dispatcher.cancel()
                    try:
                        await old_dispatcher
                    except asyncio.CancelledError:
                        pass
                    
                    new_dispatcher = asyncio.create_task(pyccp.dispatcher(rings[ring_idx]["store"]))
                    dispatcher_tasks = [(idx, d if idx != ring_idx else new_dispatcher) for idx, d in dispatcher_tasks]
                    
                    # Initialize new ring
                    await rings[ring_idx]["ports"][0].commit(0)
        
        wall_elapsed = time.time() - wall_start
        async_elapsed = asyncio.get_event_loop().time() - start_time
        
        # Clean up all dispatchers
        for ring_idx, d in dispatcher_tasks:
            d.cancel()
        
        await asyncio.gather(*[d for _, d in dispatcher_tasks], return_exceptions=True)
        
        # Report results
        message_count = commit_count[0]
        final = final_value[0]
        print(f"agent_counter_timed: {message_count} messages in {async_elapsed:.3f}s, final value = {final}")
        print(f"  ({cycle_count[0]} complete ring cycles)")
        
        return {"count": message_count, "final_value": final, "elapsed": async_elapsed, "cycles": cycle_count[0]}
    
    return asyncio.run(main())


if __name__ == "__main__":
    run()
