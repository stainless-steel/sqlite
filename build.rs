use std::env;

fn main() {
    if let Ok(features) = env::var("DEP_SQLITE3_FEATURES") {
        for feature in features.split(' ') {
            println!(r#"cargo:rustc-cfg=feature="{}""#, feature);
        }
    }
}
