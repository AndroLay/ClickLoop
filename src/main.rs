// Copyright (C) 2026 AndroLay
// SPDX-License-Identifier: GPL-3.0-only
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3 of the License.

//! ClickLoop: auto clicker ringan untuk Windows. Rust murni tanpa crate, binary kecil.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clicker;
mod config;
mod guard;
mod keys;
mod r#macro;
mod picker;
mod shell;
mod theme;
mod ui;
mod wait;
mod win32;

use std::ptr::null_mut;

const USAGE: &str = concat!(
    "ClickLoop command line\n\n",
    "--interval MS   Interval between clicks, in milliseconds\n",
    "--limit N       Stop after N clicks (0 = no limit)\n",
    "--time S        Stop after S seconds (0 = no limit)\n",
    "--jitter MS     Random +/- MS added to each interval\n",
    "--button NAME   left, right, or middle\n",
    "--profile FILE  Load a profile file before the other options\n",
    "--start         Start clicking as soon as the window opens\n",
    "--help          Show this message",
);

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    unsafe {
        if args
            .iter()
            .any(|a| matches!(a.as_str(), "--help" | "-h" | "/?"))
        {
            win32::show_error(null_mut(), USAGE);
            return;
        }
        let mut settings = config::load();
        let start = match config::apply_args(&mut settings, &args) {
            Ok(start) => start,
            Err(msg) => {
                win32::show_error(null_mut(), &format!("{msg}\n\n{USAGE}"));
                return;
            }
        };

        // Satu instance saja: dua instance akan berebut hotkey yang sama.
        let name = win32::wide("Local\\ClickLoop.SingleInstance");
        let _mutex = win32::CreateMutexW(null_mut(), 0, name.as_ptr());
        if win32::GetLastError() == win32::ERROR_ALREADY_EXISTS {
            ui::focus_existing();
            return;
        }
        if !ui::run(settings, start) {
            win32::show_error(null_mut(), "Could not create the ClickLoop window.");
        }
    }
}
