import asyncio
import pyccp
import time

def run():
    async def main():
        """
        4 agents passing incremented values in a ring, cycling for 1 second.
        
        Each cycle:
          - Create fresh ports (0-3)
          - Agent i reads port i, increments, commits to port (i+1)%4
          - Only 3 agents succeed (4th blocked by pre-fixed port[0]=0)
          - Ports all fixed → cycle complete, repeat
        
        Counts only successful commits (real message passing).
        """
        n = 4
        max_value = 1000
        commit_count = [0]
        final_value = [None]
        
        async def run_cycle():
            """One complete ring cycle: 3 commits before all ports fixed."""
            s = pyccp.Store([list(range(max_value))] * n)
            ports = [s.var(f"p{i}", i) for i in range(n)]
            
            async def agent(i):
                v = await ports[i].read()
                target = ports[(i + 1) % n]
                if not target.is_fixed():
                    await target.commit(v + 1)
                    commit_count[0] += 1
                    final_value[0] = v + 1
            
            d = asyncio.create_task(pyccp.dispatcher(s))
            await ports[0].commit(0)
            await pyccp.join(*[lambda i=i: agent(i) for i in range(n)])
            d.cancel()
            try:
                await d
            except asyncio.CancelledError:
                pass
        
        # Run cycles for 1 second
        wall_start = time.time()
        start_time = asyncio.get_event_loop().time()
        
        while asyncio.get_event_loop().time() - start_time < 1.0:
            await run_cycle()
        
        async_elapsed = asyncio.get_event_loop().time() - start_time
        
        # Report results
        message_count = commit_count[0]
        final = final_value[0]
        cycles = message_count // 3 if message_count > 0 else 0
        
        print(f"agent_counter_timed: {message_count} messages in {async_elapsed:.3f}s, final value = {final}")
        
        return {"count": message_count, "final_value": final, "elapsed": async_elapsed, "cycles": cycles}
    
    return asyncio.run(main())


if __name__ == "__main__":
    run()
