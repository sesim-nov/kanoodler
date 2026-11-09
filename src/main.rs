use std::time::Instant;
use kanoodler::solver::Solver;




fn main() {
    let start_init = Instant::now();
    let solver = Solver::new();
    println!(
        "Precomputed {} valid placements in {:?}",
        solver.get_placement_count(),
        start_init.elapsed()
    );

    // Define a starting challenge (e.g., Level 1 from the manual)
    // Use '.' for empty spaces. Spaces between characters are ignored.
    let initial_board = [
        "J J J J . . . . . . .",
        "C C B . . . . . . . .",
        "C B B . . . . . . . .",
        "C B B . . . . . . . .",
        "C . . . . . . . . . .",
    ];

    let (board_mask, piece_mask, mut current_solution) = solver
        .parse_initial_state(&initial_board)
        .expect("Failed to parse initial state");

    println!("\nStarting Puzzle State:");
    solver.print_board(&current_solution);

    let mut solutions = Vec::new();

    let start_solve = Instant::now();
    solver.solve(
        board_mask,
        piece_mask,
        &mut current_solution,
        &mut solutions,
        1,
    );

    println!("\nSolved in {:?}", start_solve.elapsed());

    if let Some(first) = solutions.first() {
        println!("Solution:");
        solver.print_board(first);
    } else {
        println!("No solution found for this configuration.");
    }
}
