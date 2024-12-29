pub fn enable_profiling() {
    puffin::set_scopes_on(true);

    let server_addr = format!("127.0.0.1:{}", puffin_http::DEFAULT_PORT);
    let server_addr_http = server_addr.to_string();
    
    match puffin_http::Server::new(&server_addr) {
        Ok(server) => {
            println!("Run this to view profiling data: puffin_viewer --url {server_addr_http}");
            std::process::Command::new("puffin_viewer")
                .arg("--url")
                .arg(&server_addr_http)
                .spawn()
                .ok();
            std::mem::forget(server);
        },
        Err(err) => {
            eprintln!("Unable to run the profiling server: {err}");
        },
    }
}
