mod process;

fn main() {
    let process_tree = process::tree::build_process_tree();
    println!("Process Tree:");
    process::tree::print_tree(&process_tree, 1, 0);
}
