mod gui;
mod process;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Linux Process Manager",
        options,
       Box::new(|_cc| Box::new(gui::MyApp::default())),
 
    )
}
