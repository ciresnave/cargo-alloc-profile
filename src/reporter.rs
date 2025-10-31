use crate::profiler::ProfileSnapshot;
use colored::*;

pub struct Reporter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Count,
    Size,
    Name,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupBy {
    Function,
    Module,
    File,
}

pub struct ReportOptions {
    pub verbosity: u8,
    pub filter: Option<String>,
    pub format: OutputFormat,
    pub min_count: Option<usize>,
    pub threshold_bytes: Option<usize>,
    pub sort_by: SortBy,
    pub limit: Option<usize>,
    pub save: Option<String>,
    pub compare: Option<String>,
    pub group_by: GroupBy,
}

impl Default for ReportOptions {
    fn default() -> Self {
        Self {
            verbosity: 0,
            filter: None,
            format: OutputFormat::Text,
            min_count: None,
            threshold_bytes: None,
            sort_by: SortBy::Count,
            limit: None,
            save: None,
            compare: None,
            group_by: GroupBy::Function,
        }
    }
}

impl Reporter {
    pub fn print_report(snapshot: ProfileSnapshot, options: ReportOptions) {
        match options.format {
            OutputFormat::Text => Self::print_text_report(snapshot, options),
            OutputFormat::Json => Self::print_json_report(snapshot, options),
        }
    }

    fn print_text_report(snapshot: ProfileSnapshot, options: ReportOptions) {
        // Handle comparison if requested
        if let Some(ref compare_file) = options.compare {
            Self::print_comparison_report(&snapshot, compare_file, &options);
            return;
        }

        // Save if requested
        if let Some(ref save_file) = options.save {
            if let Err(e) = Self::save_snapshot(&snapshot, save_file) {
                eprintln!("Warning: Failed to save profiling data: {}", e);
            }
        }

        println!("\n{}", "Allocation Profile:".bright_blue().bold());

        let sites = Self::prepare_sites(&snapshot, &options);

        for (func_name, count, total_bytes, frames) in sites.iter() {
            // Basic output: function name and count
            print!(
                "{}: {}",
                func_name.bright_white(),
                count.to_string().bright_green()
            );

            // Add verbosity levels
            if options.verbosity >= 1 {
                print!(" ({:.2} KB)", *total_bytes as f64 / 1024.0);
            }

            if options.verbosity >= 2 {
                if let Some(frame) = frames.first() {
                    print!(" [{}]", frame.dimmed());
                }
            }

            println!();

            // Show stack trace at higher verbosity
            if options.verbosity >= 3 {
                for (i, stack_frame) in frames.iter().skip(1).take(5).enumerate() {
                    println!(
                        "  {} {}",
                        if i == 0 { "└─" } else { "  " },
                        stack_frame.dimmed()
                    );
                }
                if frames.len() > 6 {
                    println!("     ... {} more frames", frames.len() - 6);
                }
            }
        }

        if sites.is_empty() {
            println!("  No allocations recorded.");
        }
    }

    fn print_json_report(snapshot: ProfileSnapshot, options: ReportOptions) {
        use serde_json::json;

        // Handle comparison if requested
        if let Some(ref compare_file) = options.compare {
            Self::print_comparison_report(&snapshot, compare_file, &options);
            return;
        }

        // Save if requested
        if let Some(ref save_file) = options.save {
            if let Err(e) = Self::save_snapshot(&snapshot, save_file) {
                eprintln!("Warning: Failed to save profiling data: {}", e);
            }
        }

        let sites = Self::prepare_sites(&snapshot, &options);
        let mut allocations = Vec::new();

        for (func_name, count, total_bytes, frames) in sites.iter() {
            let mut alloc_data = json!({
                "function": func_name,
                "count": count,
            });

            if options.verbosity >= 1 {
                alloc_data["total_bytes"] = json!(total_bytes);
            }

            if options.verbosity >= 2 {
                if let Some(frame) = frames.first() {
                    alloc_data["location"] = json!(frame);
                }
            }

            if options.verbosity >= 3 {
                alloc_data["stack_trace"] = json!(frames);
            }

            allocations.push(alloc_data);
        }

        let output = json!({
            "allocations": allocations,
            "summary": {
                "total_allocations": snapshot.total_allocations,
                "total_deallocations": snapshot.total_deallocations,
                "total_bytes_allocated": snapshot.total_bytes_allocated,
                "peak_memory": snapshot.peak_memory,
                "current_memory": snapshot.current_memory,
            }
        });

        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    }

    fn extract_function_name(frame: &str) -> String {
        // Extract just the function name without file path
        // Input: "cargo_alloc_profile::allocator::impl$0::alloc (C:\path\to\file.rs:27)"
        // Output: "cargo_alloc_profile::allocator::impl$0::alloc"

        if let Some(paren_pos) = frame.find('(') {
            frame[..paren_pos].trim().to_string()
        } else {
            frame.to_string()
        }
    }

    fn extract_module_name(frame: &str) -> String {
        // Extract module from frame
        let func_name = Self::extract_function_name(frame);
        let parts: Vec<&str> = func_name.split("::").collect();
        if parts.len() >= 2 {
            format!("{}::{}", parts[0], parts[1])
        } else if !parts.is_empty() {
            parts[0].to_string()
        } else {
            "unknown".to_string()
        }
    }

    fn extract_file_name(frame: &str) -> String {
        // Extract file path from frame
        if let Some(start) = frame.find('(') {
            if let Some(end) = frame.rfind(':') {
                let path = &frame[start + 1..end];
                // Get just the filename, not the full path
                if let Some(last_sep) = path.rfind(['\\', '/']) {
                    return path[last_sep + 1..].to_string();
                }
                return path.to_string();
            }
        }
        "unknown".to_string()
    }

    fn prepare_sites(
        snapshot: &ProfileSnapshot,
        options: &ReportOptions,
    ) -> Vec<(String, usize, usize, Vec<String>)> {
        use std::collections::HashMap;

        // Group sites based on group_by option
        let mut grouped: HashMap<String, (usize, usize, Vec<String>)> = HashMap::new();

        for site in snapshot.allocation_sites.values() {
            if let Some(frame) = site.frames.first() {
                // Determine grouping key
                let key = match options.group_by {
                    GroupBy::Function => Self::extract_function_name(frame),
                    GroupBy::Module => Self::extract_module_name(frame),
                    GroupBy::File => Self::extract_file_name(frame),
                };

                // Apply filter if specified
                if let Some(ref filter) = options.filter {
                    if !key.to_lowercase().contains(&filter.to_lowercase()) {
                        continue;
                    }
                }

                // Apply min_count filter
                if let Some(min_count) = options.min_count {
                    if site.count < min_count {
                        continue;
                    }
                }

                // Apply threshold_bytes filter
                if let Some(threshold) = options.threshold_bytes {
                    if site.total_bytes < threshold {
                        continue;
                    }
                }

                grouped
                    .entry(key)
                    .and_modify(|(count, bytes, _)| {
                        *count += site.count;
                        *bytes += site.total_bytes;
                    })
                    .or_insert((site.count, site.total_bytes, site.frames.clone()));
            }
        }

        // Convert to vec and sort
        let mut sites: Vec<(String, usize, usize, Vec<String>)> = grouped
            .into_iter()
            .map(|(name, (count, bytes, frames))| (name, count, bytes, frames))
            .collect();

        // Sort based on sort_by option
        match options.sort_by {
            SortBy::Count => sites.sort_by(|a, b| b.1.cmp(&a.1)),
            SortBy::Size => sites.sort_by(|a, b| b.2.cmp(&a.2)),
            SortBy::Name => sites.sort_by(|a, b| a.0.cmp(&b.0)),
        }

        // Apply limit if specified
        if let Some(limit) = options.limit {
            sites.truncate(limit);
        }

        sites
    }

    fn save_snapshot(snapshot: &ProfileSnapshot, path: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(snapshot)?;
        std::fs::write(path, json)?;
        eprintln!("✓ Profiling data saved to {}", path);
        Ok(())
    }

    fn print_comparison_report(
        snapshot: &ProfileSnapshot,
        compare_file: &str,
        options: &ReportOptions,
    ) {
        // Load the comparison snapshot
        let compare_snapshot = match std::fs::read_to_string(compare_file) {
            Ok(data) => match serde_json::from_str::<ProfileSnapshot>(&data) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: Failed to parse comparison file: {}", e);
                    return;
                }
            },
            Err(e) => {
                eprintln!("Error: Failed to read comparison file: {}", e);
                return;
            }
        };

        println!("\n{}", "Allocation Comparison:".bright_blue().bold());
        println!(
            "{} vs {}",
            "Current".bright_green(),
            "Baseline".bright_yellow()
        );

        // Build maps for easier comparison
        use std::collections::HashMap;
        let mut baseline_map: HashMap<String, (usize, usize)> = HashMap::new();
        for site in compare_snapshot.allocation_sites.values() {
            if let Some(frame) = site.frames.first() {
                let key = match options.group_by {
                    GroupBy::Function => Self::extract_function_name(frame),
                    GroupBy::Module => Self::extract_module_name(frame),
                    GroupBy::File => Self::extract_file_name(frame),
                };
                baseline_map
                    .entry(key)
                    .and_modify(|(count, bytes)| {
                        *count += site.count;
                        *bytes += site.total_bytes;
                    })
                    .or_insert((site.count, site.total_bytes));
            }
        }

        let current_sites = Self::prepare_sites(snapshot, options);

        for (name, current_count, current_bytes, _frames) in current_sites.iter() {
            if let Some((baseline_count, baseline_bytes)) = baseline_map.get(name) {
                let count_diff = *current_count as isize - *baseline_count as isize;
                let bytes_diff = *current_bytes as isize - *baseline_bytes as isize;

                let count_str = if count_diff > 0 {
                    format!("+{}", count_diff).bright_red()
                } else if count_diff < 0 {
                    format!("{}", count_diff).bright_green()
                } else {
                    "±0".normal()
                };

                let bytes_str = if bytes_diff > 0 {
                    format!("(+{:.2} KB)", bytes_diff as f64 / 1024.0).bright_red()
                } else if bytes_diff < 0 {
                    format!("({:.2} KB)", bytes_diff as f64 / 1024.0).bright_green()
                } else {
                    "(±0 KB)".normal()
                };

                println!(
                    "{}: {} → {} {} {}",
                    name.bright_white(),
                    baseline_count,
                    current_count,
                    count_str,
                    bytes_str
                );
            } else {
                // New allocation site
                println!(
                    "{}: {} {} {}",
                    name.bright_white(),
                    format!("{}", current_count).bright_green(),
                    "[NEW]".bright_yellow(),
                    format!("({:.2} KB)", *current_bytes as f64 / 1024.0)
                );
            }
        }

        // Show removed allocation sites
        for (name, (baseline_count, baseline_bytes)) in baseline_map.iter() {
            if !current_sites.iter().any(|(n, _, _, _)| n == name) {
                println!(
                    "{}: {} {} {}",
                    name.dimmed(),
                    baseline_count,
                    "[REMOVED]".bright_cyan(),
                    format!("({:.2} KB)", *baseline_bytes as f64 / 1024.0).dimmed()
                );
            }
        }
    }
}
