fn main() {
    // Tell Cargo where to find libraries
    println!("cargo:rustc-link-search=native=./lib");
    //println!("cargo::rustc-link-arg=-Wl,-rpath,/home/aaron/Projects/zinnia/lib");
    println!("cargo:rustc-link-lib=dylib=vosk");
}
