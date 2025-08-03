// ABOUTME: Debug script to test Macroquad coordinate system behavior
// ABOUTME: Creates a simple test pattern to verify Y-axis orientation

use macroquad::prelude::*;

#[macroquad::main("Coordinate System Test")]
async fn main() {
    loop {
        clear_background(BLACK);

        // Draw a test pattern to understand coordinate system
        // Top-left corner (should be at screen top-left)
        draw_rectangle(10.0, 10.0, 50.0, 50.0, RED);
        draw_text("TOP-LEFT", 15.0, 35.0, 20.0, WHITE);

        // Bottom-right corner (should be at screen bottom-right)
        let screen_w = screen_width();
        let screen_h = screen_height();
        draw_rectangle(screen_w - 60.0, screen_h - 60.0, 50.0, 50.0, BLUE);
        draw_text(
            "BOTTOM-RIGHT",
            screen_w - 120.0,
            screen_h - 35.0,
            20.0,
            WHITE,
        );

        // Draw a grid to show coordinate progression
        for i in 0..10 {
            let x = i as f32 * 50.0;
            let y = i as f32 * 30.0;
            draw_circle(x + 100.0, y + 100.0, 5.0, GREEN);
            draw_text(&format!("({},{})", i, i), x + 110.0, y + 105.0, 16.0, WHITE);
        }

        // Instructions
        draw_text(
            "Macroquad Coordinate System Test",
            10.0,
            screen_h - 20.0,
            20.0,
            YELLOW,
        );
        draw_text(
            "Y should increase DOWNWARD (standard graphics)",
            10.0,
            screen_h - 40.0,
            16.0,
            LIGHTGRAY,
        );

        next_frame().await;
    }
}
