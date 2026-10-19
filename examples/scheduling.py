from pyccp import Store

def run():
    """Solve job scheduling: assign 8 jobs to 3 machines, minimize makespan."""
    job_durations = [5, 3, 4, 6, 2, 7, 3, 1]  # 8 jobs
    n_jobs = len(job_durations)
    n_machines = 3
    
    # Variables: machine[i] in 0..n_machines-1 — job i assigned to which machine
    s = Store([list(range(n_machines)) for _ in range(n_jobs)])
    
    # Objective: minimize makespan = max(total time on each machine)
    # Compute makespan externally (no LinearEq for this complexity)
    
    # Enumerate all assignments
    sols = s.solve_all(limit=3**n_jobs)  # 3^8 = 6561 possibilities
    
    best_makespan = float('inf')
    best_assign = None
    for sol in sols:
        machine_times = [0] * n_machines
        for job_idx, machine_idx in enumerate(sol):
            machine_times[machine_idx] += job_durations[job_idx]
        makespan = max(machine_times)
        if makespan < best_makespan:
            best_makespan = makespan
            best_assign = sol
    
    assert best_assign is not None
    assert best_makespan <= 16  # Known bound
    print(f"scheduling: assignment={best_assign}, makespan={best_makespan}")
    return (best_assign, best_makespan)
