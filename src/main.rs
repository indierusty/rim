mod buffer;
mod rim;

use rim::Rim;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.len() == 0 {
        println!("Usage: rim [file_path]");
        return;
    }

    Rim::new(&args[0]).run().unwrap();
}
