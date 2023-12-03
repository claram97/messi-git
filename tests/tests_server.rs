use messi::server;
use std::io;
const PORT: &str = "9418";

#[test]
#[ignore = "This test only works if the server is running"]
fn test_run_server() -> io::Result<()> {
    server::run("localhost", PORT, "/home/rgestoso/daemon/server", ".git")
}
