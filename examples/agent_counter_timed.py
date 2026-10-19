import asyncio
import pyccp

def run():
    async def main():
        """
        Ring agents passing incremented values for 1 second.
        
        Each agent:
        1. Reads a value from its port
        2. Increments it
        3. Commits the incremented value to the next agent's port
        
        The ring continuously passes messages back and forth, with each
        pass incrementing the value. After 1 second, reports total messages
        and the final value.
        
        Note: Due to CCP v1 semantics (final state observation only),
        agents run in a finite sequence, not continuous concurrency.
        """
        n = 4
        max_passes = 1000  # Upper bound on iterations
        s = pyccp.Store([list(range(max_passes))] * n)
        ports = [s.var(f"p{i}", i) for i in range(n)]
        counter = []
        final_value = [None]
        
        async def agent(i):
            """Agent reads from port i, increments value, commits to next port."""
            v = await ports[i].read()
            counter.append(i)
            final_value[0] = v
            target = ports[(i + 1) % n]
            if not target.is_fixed():
                await target.commit(v + 1)

        d = asyncio.create_task(pyccp.dispatcher(s))
        
        # Start the ring with initial value 0
        await ports[0].commit(0)
        
        # Run agents repeatedly in the ring for 1 second
        start_time = asyncio.get_event_loop().time()
        iteration = 0
        
        while asyncio.get_event_loop().time() - start_time < 1.0:
            try:
                # Run one complete pass around the ring
                await pyccp.join(*[lambda i=i: agent(i) for i in range(n)])
                iteration += 1
            except Exception:
                break
        
        # Clean up dispatcher
        d.cancel()
        try:
            await d
        except asyncio.CancelledError:
            pass
        
        # Report results
        message_count = len(counter)
        final = final_value[0]
        print(f"agent_counter_timed: {message_count} messages in 1.0s, final value = {final}")
        
        return {"count": message_count, "final_value": final}
    
    return asyncio.run(main())
