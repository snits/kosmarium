// SPDX-License-Identifier: MIT
// Copyright (c) 2025 Jerry Snitselaar and contributors

// ABOUTME: Application implementations - different ways to use the simulation engine
// ABOUTME: Demonstrates engine flexibility through specialized application instances

pub mod terrain_explorer;
pub mod weather_demo;

// Re-export application entry points
pub use weather_demo::run_weather_demo;
