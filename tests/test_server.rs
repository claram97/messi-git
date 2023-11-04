use std::io;

use messi::server;
const PORT: &str = "9418";

#[test]
#[ignore]
fn test_run_server() -> io::Result<()> {
    server::run("localhost", PORT, "/home/rgestoso/daemon/server", ".git")
}

