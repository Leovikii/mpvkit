use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
    time::{Duration, Instant},
};
use tauri_plugin_updater::{Error, UpdaterExt};

const PAYLOAD: &[u8] = include_bytes!("../../../scripts/fixtures/updater/payload.txt");
const SIGNATURE: &str = include_str!("../../../scripts/fixtures/updater/payload.txt.sig");

// Only the public key and a signed, non-executable text file are checked in.
// HTTP is limited to this loopback fixture; the production endpoints require HTTPS.
#[test]
fn plugin_download_verifies_bytes_and_rejects_network_and_signature_failures() {
    let mut context = tauri::test::mock_context(tauri::test::noop_assets());
    context.config_mut().plugins.0.insert(
        "updater".into(),
        serde_json::json!({
            "pubkey": include_str!("../../../scripts/fixtures/updater/public-key.txt").trim(),
            "dangerousInsecureTransportProtocol": true
        }),
    );
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .build(context)
        .unwrap();

    for case in [
        "valid",
        "corrupt",
        "bad-signature",
        "download-500",
        "truncated",
        "metadata-500",
    ] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let manifest = serde_json::to_vec(&serde_json::json!({
            "version": "9.0.0",
            "notes": "Download verification only",
            "platforms": {(if cfg!(target_os = "windows") { "windows-x86_64" } else { "linux-x86_64" }): {
                "url": format!("{origin}/payload"),
                "signature": if case == "bad-signature" { "invalid signature" } else { SIGNATURE.trim() }
            }}
        })).unwrap();
        let server = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(10);
            let requests = if case == "metadata-500" { 1 } else { 2 };
            for _ in 0..requests {
                let mut stream = loop {
                    match listener.accept() {
                        Ok((stream, _)) => break stream,
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            assert!(Instant::now() < deadline, "{case}: missing HTTP request");
                            thread::sleep(Duration::from_millis(5));
                        }
                        Err(error) => panic!("{case}: {error}"),
                    }
                };
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                stream
                    .set_write_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut request = Vec::new();
                while !request.windows(4).any(|v| v == b"\r\n\r\n") {
                    let mut buffer = [0; 1024];
                    let count = stream.read(&mut buffer).unwrap();
                    assert!(count > 0 && request.len() + count <= 4096);
                    request.extend_from_slice(&buffer[..count]);
                }
                let metadata = request.starts_with(b"GET /manifest ");
                assert!(metadata || request.starts_with(b"GET /payload "));
                let failed =
                    (metadata && case == "metadata-500") || (!metadata && case == "download-500");
                let mut body = if metadata {
                    manifest.clone()
                } else {
                    PAYLOAD.to_vec()
                };
                if !metadata && case == "corrupt" {
                    body[0] ^= 1;
                }
                let length = body.len();
                if !metadata && case == "truncated" {
                    body.truncate(length / 2);
                }
                let status = if failed {
                    "500 Internal Server Error"
                } else {
                    "200 OK"
                };
                write!(stream, "HTTP/1.1 {status}\r\nContent-Length: {length}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n").unwrap();
                stream.write_all(&body).unwrap();
            }
        });

        tauri::async_runtime::block_on(async {
            let checked = app
                .updater_builder()
                .endpoints(vec![format!("{origin}/manifest").parse().unwrap()])
                .unwrap()
                .no_proxy()
                .timeout(Duration::from_secs(3))
                .build()
                .unwrap()
                .check()
                .await;
            if case == "metadata-500" {
                assert!(checked.is_err());
                return;
            }
            let update = checked.unwrap().expect("newer signed fixture");
            assert_eq!(update.version, "9.0.0");
            // Never call install: this verifies the real plugin's HTTP/signature path only.
            let downloaded = update.download(|_, _| {}, || {}).await;
            match case {
                "valid" => assert_eq!(downloaded.unwrap(), PAYLOAD),
                "corrupt" => assert!(matches!(downloaded, Err(Error::Minisign(_)))),
                "bad-signature" => assert!(matches!(downloaded, Err(Error::Base64(_)))),
                "download-500" => assert!(matches!(downloaded, Err(Error::Network(_)))),
                "truncated" => assert!(matches!(downloaded, Err(Error::Reqwest(_)))),
                _ => unreachable!(),
            }
        });
        server.join().unwrap();
    }
}
