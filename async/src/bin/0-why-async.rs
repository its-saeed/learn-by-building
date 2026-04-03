// Lesson 0: Why Async?
// Benchmark: threads vs async tasks

fn main() {
    // TODO:
    // 1. Spawn 10,000 std::thread that each sleep for 1 second
    // 2. Measure wall time and peak memory
    // 3. Print results
    //
    // Then compare with async version (separate binary or flag):
    // 1. Spawn 10,000 tokio tasks that each tokio::time::sleep for 1 second
    // 2. Measure wall time and peak memory
    // 3. Print results
    //
    // Expected: threads use ~80GB virtual memory, async uses ~10MB
    todo!()
}
