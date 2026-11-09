use kanoodler::solver::Solver;
use kanoodler::board::*;
use eframe::egui;

// GUI helpers for dragging and rotating
fn rotate(coords: &[Coord]) -> Vec<Coord> {
    let rotated: Vec<Coord> = coords.iter().map(|&(r, c)| (c, -r)).collect();
    normalize(&rotated)
}

fn flip(coords: &[Coord]) -> Vec<Coord> {
    let flipped: Vec<Coord> = coords.iter().map(|&(r, c)| (r, -c)).collect();
    normalize(&flipped)
}

fn normalize(coords: &[Coord]) -> Vec<Coord> {
    let min_r = coords.iter().map(|&(r, _)| r).min().unwrap_or(0);
    let min_c = coords.iter().map(|&(_, c)| c).min().unwrap_or(0);
    let mut normalized: Vec<Coord> = coords.iter().map(|&(r, c)| (r - min_r, c - min_c)).collect();
    normalized.sort();
    normalized
}

// Maps piece indices to the exact reference colors
fn get_piece_color_by_idx(piece: usize) -> egui::Color32 {
    match piece {
        0 => egui::Color32::from_rgb(255, 127, 0),   // A Orange
        1 => egui::Color32::from_rgb(220, 20, 60),   // B Red
        2 => egui::Color32::from_rgb(0, 0, 139),     // C Dark Blue
        3 => eframe::egui::Color32::from_rgb(255, 182, 193), // D Light Pink
        4 => eframe::egui::Color32::from_rgb(0, 100, 0),     // E Dark Green
        5 => eframe::egui::Color32::from_rgb(240, 240, 240), // F White
        6 => eframe::egui::Color32::from_rgb(173, 216, 230), // G Light Blue
        7 => eframe::egui::Color32::from_rgb(255, 105, 180), // H Bright Pink
        8 => eframe::egui::Color32::from_rgb(255, 215, 0),   // I Yellow
        9 => eframe::egui::Color32::from_rgb(128, 0, 128),   // J Purple
        10 => eframe::egui::Color32::from_rgb(144, 238, 144),// K Light Green
        11 => eframe::egui::Color32::from_rgb(128, 128, 128),// L Grey
        _ => eframe::egui::Color32::TRANSPARENT,
    }
}

#[derive(Clone)]
struct PlacedPiece {
    piece_idx: usize,
    shape: Vec<Coord>,
    r: i32,
    c: i32,
}

// --- EGUI APPLICATION ---

struct KanoodleApp {
    solver: Solver,
    status: String,
    placed_pieces: Vec<PlacedPiece>,
    dragging: Option<(usize, Vec<Coord>)>,
}

impl Default for KanoodleApp {
    fn default() -> Self {
        Self {
            solver: Solver::new(),
            status: "Ready. Drag pieces onto the board, press R to rotate, F to flip.".to_string(),
            placed_pieces: Vec::new(),
            dragging: None,
        }
    }
}

impl eframe::App for KanoodleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut remove_placed = None;

        // Handle keyboard input for dragged piece
        if let Some((_, ref mut shape)) = self.dragging {
            if ctx.input(|i| i.key_pressed(egui::Key::R)) {
                *shape = rotate(shape);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::F)) {
                *shape = flip(shape);
            }
        }

        egui::SidePanel::left("controls_panel").default_width(220.0).show(ctx, |ui| {
            ui.heading("Kanoodle Solver");
            ui.add_space(10.0);

            if ui.button("Solve Board").clicked() {
                //let start_solve = Instant::now();
                
                // Build bitmasks dynamically from the visually placed pieces
                let mut board_mask = 0u64;
                let mut piece_mask = 0u16;
                let mut current_solution = Vec::new();

                for pp in &self.placed_pieces {
                    piece_mask |= 1 << pp.piece_idx;
                    let mut p_mask = 0u64;
                    for &(sr, sc) in &pp.shape {
                        let cell_idx = ((pp.r + sr) as usize) * WIDTH + ((pp.c + sc) as usize);
                        p_mask |= 1u64 << cell_idx;
                    }
                    board_mask |= p_mask;
                    
                    // Match visual placement to exact solver placement ID
                    // (Requires access to solver.placements or doing it visually)
                    // For now, let's just let the solver fill the rest!
                    // If we can't seed `current_solution`, the GUI won't highlight pre-placed. 
                    // Assuming solver handles pre-seeded masks perfectly, it will just fill the blanks.
                }

                let mut solutions = Vec::new();
                self.solver.solve(board_mask, piece_mask, &mut current_solution, &mut solutions, 1);
                
                if let Some(first) = solutions.first() {
                    self.status = format!("Solved!");
                    
                    // The solver returns indices of newly placed pieces. We parse them and add to GUI.
                    for &p_idx in first {
                        let p = self.solver.get_placement(p_idx); // uses their library method
                        
                        let mut r_min = 100;
                        let mut c_min = 100;
                        for cell in 0..NUM_CELLS {
                            if (p.get_mask() & (1u64 << cell)) != 0 {
                                let r = (cell / WIDTH) as i32;
                                let c = (cell % WIDTH) as i32;
                                if r < r_min { r_min = r; }
                                if c < c_min { c_min = c; }
                            }
                        }
                        
                        let mut shape = Vec::new();
                        for cell in 0..NUM_CELLS {
                            if (p.get_mask() & (1u64 << cell)) != 0 {
                                shape.push(((cell / WIDTH) as i32 - r_min, (cell % WIDTH) as i32 - c_min));
                            }
                        }
                        shape.sort();
                        
                        self.placed_pieces.push(PlacedPiece {
                            piece_idx: PIECE_LABELS.iter().position(|&l| l == p.get_label()).unwrap(),
                            shape,
                            r: r_min,
                            c: c_min,
                        });
                    }
                } else {
                    self.status = "No solution found. Check your pieces!".to_string();
                }
            }

            if ui.button("Clear Board").clicked() {
                self.placed_pieces.clear();
                self.dragging = None;
                self.status = "Cleared.".to_string();
            }

            ui.add_space(20.0);
            ui.label(&self.status);
            
            ui.add_space(20.0);
            ui.label("Tray (Click/Drag to board):");
            
            let tray_cell_size = 15.0;
            
            // Render unplaced pieces in the tray
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for piece_idx in 0..NUM_PIECES {
                        let is_placed = self.placed_pieces.iter().any(|p| p.piece_idx == piece_idx);
                        let is_dragging = self.dragging.as_ref().map_or(false, |(id, _)| *id == piece_idx);
                        
                        if !is_placed && !is_dragging {
                            let (response, painter) = ui.allocate_painter(
                                egui::vec2(tray_cell_size * 5.0, tray_cell_size * 4.0),
                                egui::Sense::click_and_drag(),
                            );
                            
                            let base = response.rect.min;
                            let color = get_piece_color_by_idx(piece_idx);
                            
                            for &(r, c) in PIECE_SHAPES[piece_idx] {
                                let rect = egui::Rect::from_min_max(
                                    base + egui::vec2(c as f32 * tray_cell_size, r as f32 * tray_cell_size),
                                    base + egui::vec2((c + 1) as f32 * tray_cell_size, (r + 1) as f32 * tray_cell_size)
                                );
                                painter.rect_filled(rect.shrink(1.0), 2.0, color);
                            }
                            
                            if response.drag_started() || response.clicked() {
                                self.dragging = Some((piece_idx, PIECE_SHAPES[piece_idx].to_vec()));
                            }
                        }
                    }
                });
            });
        });

        // Main drag-and-drop interactive canvas
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(
                ui.available_size(),
                egui::Sense::click(),
            );

            let board_rect = response.rect;
            let cell_size = 40.0;
            let padding = 5.0;

            let start_x = board_rect.center().x - (WIDTH as f32 * cell_size) / 2.0;
            let start_y = board_rect.center().y - (HEIGHT as f32 * cell_size) / 2.0;

            // 1. Draw empty board cells
            for r in 0..HEIGHT {
                for c in 0..WIDTH {
                    let min = egui::pos2(
                        start_x + c as f32 * cell_size,
                        start_y + r as f32 * cell_size,
                    );
                    let max = egui::pos2(min.x + cell_size - padding, min.y + cell_size - padding);
                    let rect = egui::Rect::from_min_max(min, max);
                    painter.rect_filled(rect, 4.0, egui::Color32::from_gray(40)); 
                }
            }
            
            // 2. Draw placed pieces
            for (idx, pp) in self.placed_pieces.iter().enumerate() {
                let color = get_piece_color_by_idx(pp.piece_idx);
                let mut piece_rect = egui::Rect::NOTHING;
                
                for &(r, c) in &pp.shape {
                    let min = egui::pos2(
                        start_x + (pp.c + c) as f32 * cell_size,
                        start_y + (pp.r + r) as f32 * cell_size,
                    );
                    let max = egui::pos2(min.x + cell_size - padding, min.y + cell_size - padding);
                    let rect = egui::Rect::from_min_max(min, max);
                    piece_rect = piece_rect.union(rect);
                    painter.rect_filled(rect, 4.0, color); 
                }
                
                // Allow picking pieces back up off the board
                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        if piece_rect.contains(pos) {
                            remove_placed = Some(idx);
                        }
                    }
                }
            }
            
            // 3. Draw active dragging piece snapping to cursor
            if let Some((piece_idx, ref shape)) = self.dragging {
                if let Some(pos) = ctx.pointer_hover_pos() {
                    let color = get_piece_color_by_idx(piece_idx);
                    
                    // Snap cursor coordinates to grid positions
                    let grid_c = ((pos.x - start_x) / cell_size).round() as i32;
                    let grid_r = ((pos.y - start_y) / cell_size).round() as i32;
                    
                    for &(r, c) in shape {
                        let draw_c = grid_c + c;
                        let draw_r = grid_r + r;
                        
                        let min = egui::pos2(
                            start_x + draw_c as f32 * cell_size,
                            start_y + draw_r as f32 * cell_size,
                        );
                        let max = egui::pos2(min.x + cell_size - padding, min.y + cell_size - padding);
                        let rect = egui::Rect::from_min_max(min, max);
                        
                        // Make piece semi-transparent while dragging
                        let mut preview_color = color;
                        preview_color[3] = 120; // set alpha
                        painter.rect_filled(rect, 4.0, preview_color); 
                    }
                    
                    // 4. Handle dropping the piece on the board
                    if ctx.input(|i| i.pointer.primary_released()) {
                        let mut valid = true;
                        let mut p_mask = 0u64;
                        
                        for &(r, c) in shape {
                            let br = grid_r + r;
                            let bc = grid_c + c;
                            if br < 0 || br >= HEIGHT as i32 || bc < 0 || bc >= WIDTH as i32 {
                                valid = false;
                                break;
                            }
                            let cell_idx = (br as usize) * WIDTH + (bc as usize);
                            p_mask |= 1u64 << cell_idx;
                        }
                        
                        // Check piece overlap
                        if valid {
                            for pp in &self.placed_pieces {
                                for &(r, c) in &pp.shape {
                                    let cell_idx = ((pp.r + r) as usize) * WIDTH + ((pp.c + c) as usize);
                                    if (p_mask & (1u64 << cell_idx)) != 0 {
                                        valid = false;
                                    }
                                }
                            }
                        }
                        
                        if valid {
                            self.placed_pieces.push(PlacedPiece {
                                piece_idx,
                                shape: shape.clone(),
                                r: grid_r,
                                c: grid_c,
                            });
                        }
                        self.dragging = None;
                    }
                }
            }
        });
        
        if let Some(idx) = remove_placed {
            let pp = self.placed_pieces.remove(idx);
            self.dragging = Some((pp.piece_idx, pp.shape));
        }
    }
}

// ---------------------------------------------
// NATIVE (Desktop) ENTRY POINT
// ---------------------------------------------
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([900.0, 500.0])
            .with_title("Kanoodle Drag & Drop Solver"),
        ..Default::default()
    };
    eframe::run_native(
        "Kanoodle Solver",
        options,
        Box::new(|_cc| Box::new(KanoodleApp::default())),
    )
}

// ---------------------------------------------
// WEB (Wasm) ENTRY POINT
// ---------------------------------------------
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect panics to the browser console so you can see errors
    console_error_panic_hook::set_once();

    let web_options = eframe::WebOptions::default();

    // The new eframe web runner requires an async execution environment
    wasm_bindgen_futures::spawn_local(async {
        let runner = eframe::WebRunner::new();
        runner
            .start(
                "the_canvas_id", 
                web_options, 
                Box::new(|_cc| Box::new(KanoodleApp::default())), // Matches your desktop setup
            )
            .await
            .expect("failed to start eframe");
    });
}