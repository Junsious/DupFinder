#![windows_subsystem = "windows"]

// Import necessary modules and crates
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::sync::{Arc, Mutex, mpsc};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;
use rayon::prelude::*;
use eframe::{egui, App, Frame};
use rfd::FileDialog;

// Function to hash a file using SHA-256
fn hash_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = File::open(path)?; // Attempt to open the file
    let mut hasher = Sha256::new(); // Create a new SHA-256 hasher
    let mut buffer = vec![0; 4096]; // Buffer to hold file data

    // Read the file in chunks and update the hasher
    while let Ok(bytes_read) = file.read(&mut buffer) {
        if bytes_read == 0 {
            break; // Break the loop if no more bytes are read
        }
        hasher.update(&buffer[..bytes_read]); // Update the hasher with the read bytes
    }

    // Return the final hash in hexadecimal format
    Ok(format!("{:x}", hasher.finalize()))
}

// Function to find duplicate files in a directory (using multithreading)
fn find_duplicates(
    dir: &str,
    progress: Arc<Mutex<f32>>,
    stop_receiver: Arc<Mutex<mpsc::Receiver<()>>>,
) -> io::Result<HashMap<String, Vec<String>>> {
    // Collect all files in the directory and its subdirectories
    let entries: Vec<_> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| entry.file_type().is_file())
        .collect();

    let total_files = entries.len(); // Total number of files to be processed
    let file_map: Arc<Mutex<HashMap<String, Vec<String>>>> = Arc::new(Mutex::new(HashMap::new())); // To store hashes and their corresponding file paths

    // Process each file in parallel
    entries.par_iter().enumerate().for_each(|(i, entry)| {
        // Check for a stop signal
        if stop_receiver.lock().unwrap().try_recv().is_ok() {
            return; // If a stop signal is received, exit
        }

        let path = entry.path().to_path_buf(); // Get the path of the current entry
        if let Ok(hash) = hash_file(&path) { // Hash the file
            // Update progress
            let mut progress = progress.lock().unwrap();
            *progress = (i + 1) as f32 / total_files as f32; // Update progress percentage

            // Update the file_map with the hash and corresponding file path
            let mut file_map = file_map.lock().unwrap();
            file_map.entry(hash).or_insert_with(Vec::new).push(path.display().to_string());
        }
    });

    // Filter out the duplicates from the file_map
    let duplicates = {
        let file_map = file_map.lock().unwrap();
        file_map.iter()
            .filter(|(_, v)| v.len() > 1) // Keep only hashes with multiple files
            .map(|(k, v)| (k.clone(), v.clone())) // Collect duplicates
            .collect::<HashMap<_, _>>() // Collect as a HashMap
    };

    Ok(duplicates) // Return the duplicates
}

// Application structure for the UI to find duplicates
struct DuplicateFinderApp {
    dir_to_scan: String, // Directory selected for scanning
    duplicates: Arc<Mutex<HashMap<String, Vec<String>>>>, // Map to hold duplicates
    progress: Arc<Mutex<f32>>, // Progress of the scanning process
    searching: bool, // Flag to indicate if a search is in progress
    stop_sender: Option<mpsc::Sender<()>>, // Sender for stopping the search
    stop_receiver: Arc<Mutex<mpsc::Receiver<()>>>, // Receiver for stopping the search
}

// Default implementation for the DuplicateFinderApp
impl Default for DuplicateFinderApp {
    fn default() -> Self {
        let (stop_sender, stop_receiver) = mpsc::channel(); // Create a channel for stopping the process
        Self {
            dir_to_scan: String::new(), // Initialize directory to scan
            duplicates: Arc::new(Mutex::new(HashMap::new())), // Initialize duplicates map
            progress: Arc::new(Mutex::new(0.0)), // Initialize progress to 0
            searching: false, // Searching is initially false
            stop_sender: Some(stop_sender), // Store the sender for stopping the process
            stop_receiver: Arc::new(Mutex::new(stop_receiver)), // Store the receiver for stopping the process
        }
    }
}

// Implement the App trait for DuplicateFinderApp
impl App for DuplicateFinderApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut Frame) {
        // Central panel for UI elements
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Select a directory to scan:"); // Label for directory selection

            // Button to choose a directory
            if ui.button("Choose Directory").clicked() {
                if let Some(path) = FileDialog::new().pick_folder() { // Open file dialog to pick a folder
                    self.dir_to_scan = path.display().to_string(); // Update the directory to scan
                    self.duplicates.lock().unwrap().clear(); // Clear previous duplicates
                }
            }

            ui.label(format!("Current Directory: {}", self.dir_to_scan)); // Display the selected directory

            // Button to start the search if conditions are met
            if !self.dir_to_scan.is_empty() && !self.searching && ui.button("Start Search").clicked() {
                self.searching = true; // Set searching flag to true
                let dir_to_scan = self.dir_to_scan.clone(); // Clone the directory path
                let progress = Arc::clone(&self.progress); // Clone the progress Arc
                let duplicates = Arc::clone(&self.duplicates); // Clone the duplicates Arc
                let stop_receiver = Arc::clone(&self.stop_receiver); // Clone the stop receiver Arc

                // Spawn a new thread for the search process
                std::thread::spawn(move || {
                    let found = find_duplicates(&dir_to_scan, progress, stop_receiver).unwrap_or_default(); // Find duplicates
                    let mut duplicates = duplicates.lock().unwrap(); // Lock and update duplicates
                    *duplicates = found; // Store found duplicates
                });
            }

            // Button to stop the search if it's in progress
            if self.searching && ui.button("Stop Search").clicked() {
                if let Some(sender) = &self.stop_sender {
                    let _ = sender.send(()); // Send stop signal
                    self.searching = false; // Immediately stop the search
                    *self.progress.lock().unwrap() = 0.0; // Reset progress to 0
                }
            }

            // Progress bar display
            if self.searching {
                ui.add(egui::ProgressBar::new(*self.progress.lock().unwrap()).animate(true)
                    .desired_height(24.0)); // Increase height of the progress bar
                if *self.progress.lock().unwrap() >= 1.0 {
                    self.searching = false; // Stop searching if progress is complete
                    *self.progress.lock().unwrap() = 0.0; // Reset progress
                }
            } else {
                // If not searching, disable progress bar animation
                ui.add(egui::ProgressBar::new(0.0).desired_height(24.0));
            }

            // Display found duplicates
            let duplicates_map = self.duplicates.lock().unwrap(); // Lock and retrieve duplicates map
            if !duplicates_map.is_empty() {
                ui.heading("Found Duplicates:"); // Heading for duplicates section
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (hash, files) in duplicates_map.iter() { // Iterate over found duplicates
                        ui.collapsing(format!("Hash: {}", hash), |ui| {
                            for file in files { // List each file under the corresponding hash
                                ui.horizontal(|ui| {
                                    ui.label(file); // Display file path
                                });
                            }
                        });
                    }
                });
            }
        });
    }
}

// Entry point for the application
fn main() -> Result<(), eframe::Error> {
    let app = DuplicateFinderApp::default(); // Create a new instance of the app
    let native_options = eframe::NativeOptions::default(); // Default native options for the app
    eframe::run_native(
        "DupFinder", // Window title
        native_options, // Native options
        Box::new(|_| Ok(Box::new(app))), // Create the app instance
    )
}
