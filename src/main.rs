use std::time::Instant;
use kanoodler::solver::Solver;
use kanoodler::board::*;
use eframe::egui;

// --- EGUI APPLICATION ---

struct KanoodleApp {
    solver: Solver,
    grid: [[Option<char>; WIDTH]; HEIGHT],
    status: String,
}

impl Default for KanoodleApp {
    fn default() -> Self {
        Self {
            solver: Solver::new(),
            grid: [[None; WIDTH]; HEIGHT],
            status: "Ready. Click 'Solve' to find a solution.".to_string(),
        }
    }
}

impl eframe::App for KanoodleApp {
    // In newer eframe versions, the trait method is 'ui' and it passes an egui::Ui
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Sidebar for controls and piece tray
        egui::Panel::left("controls_panel").show(ui, |ui| {
            ui.heading("Kanoodle Solver");
            ui.add_space(10.0);

            if ui.button("Solve Empty Board").clicked() {
                let start_solve = Instant::now();
                let mut current_solution = Vec::new();
                let mut solutions = Vec::new();
                
                self.solver.solve(0, 0, &mut current_solution, &mut solutions, 1);
                
                if let Some(first) = solutions.first() {
                    self.status = format!("Solved in {:?}", start_solve.elapsed());
                    
                    // Map solution back to the 2D grid state
                    for &p_idx in first {
                        let p = self.solver.get_placement(p_idx);
                        let label = p.get_label();
                        for cell in 0..NUM_CELLS {
                            if (p.get_mask() & (1u64 << cell)) != 0 {
                                let r = cell / WIDTH;
                                let c = cell % WIDTH;
                                self.grid[r][c] = Some(label);
                            }
                        }
                    }
                } else {
                    self.status = "No solution found.".to_string();
                }
            }

            if ui.button("Clear Board").clicked() {
                self.grid = [[None; WIDTH]; HEIGHT];
                self.status = "Cleared.".to_string();
            }

            ui.add_space(20.0);
            ui.label(&self.status);
            
            ui.add_space(20.0);
            ui.label("Tray (Drag & Drop Coming Soon):");
            // Placeholder for Tray UI
        });

        // Main area for rendering the board
        egui::CentralPanel::default().show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(
                ui.available_size(),
                egui::Sense::click(),
            );

            let board_rect = response.rect;
            let cell_size = 40.0;
            let padding = 5.0;

            // Center the board visually
            let start_x = board_rect.center().x - (WIDTH as f32 * cell_size) / 2.0;
            let start_y = board_rect.center().y - (HEIGHT as f32 * cell_size) / 2.0;

            for r in 0..HEIGHT {
                for c in 0..WIDTH {
                    let min = egui::pos2(
                        start_x + c as f32 * cell_size,
                        start_y + r as f32 * cell_size,
                    );
                    let max = egui::pos2(min.x + cell_size - padding, min.y + cell_size - padding);
                    let rect = egui::Rect::from_min_max(min, max);

                    // Determine cell color
                    let color = match self.grid[r][c] {
                        Some(piece) => get_piece_color(piece),
                        None => egui::Color32::from_gray(40), // Empty cell
                    };

                    painter.rect_filled(rect, 4.0, color); 
                    painter.rect_stroke(rect, 4.0, egui::Stroke::new(1.0, egui::Color32::BLACK), egui::StrokeKind::Middle);
                }
            }
        });
    }
}

// Maps piece labels to the exact reference colors
fn get_piece_color(piece: char) -> egui::Color32 {
    match piece {
        'A' => egui::Color32::from_rgb(255, 127, 0),   // Orange
        'B' => egui::Color32::from_rgb(220, 20, 60),   // Red
        'C' => egui::Color32::from_rgb(0, 0, 139),     // Dark Blue
        'D' => egui::Color32::from_rgb(255, 182, 193), // Light Pink
        'E' => egui::Color32::from_rgb(0, 100, 0),     // Dark Green
        'F' => egui::Color32::from_rgb(240, 240, 240), // White
        'G' => egui::Color32::from_rgb(173, 216, 230), // Light Blue
        'H' => egui::Color32::from_rgb(255, 105, 180), // Bright Pink
        'I' => egui::Color32::from_rgb(255, 215, 0),   // Yellow
        'J' => egui::Color32::from_rgb(128, 0, 128),   // Purple
        'K' => egui::Color32::from_rgb(144, 238, 144), // Light Green
        'L' => egui::Color32::from_rgb(128, 128, 128), // Grey
        _ => egui::Color32::TRANSPARENT,
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 400.0])
            .with_title("Kanoodle Solver"),
        ..Default::default()
    };
    eframe::run_native(
        "Kanoodle Solver",
        options,
        Box::new(|_cc| Ok(Box::<KanoodleApp>::default())),
    )
}