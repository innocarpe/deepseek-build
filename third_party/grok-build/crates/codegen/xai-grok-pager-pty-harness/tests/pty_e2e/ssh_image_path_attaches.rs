// Per-test-case module for the `pty_e2e_clipboard` integration test crate.
//
// An image path pasted into a pane on a **remote** host must attach, exactly as
// it does locally. The path names a file on the host the pager runs on, so the
// host under test can resolve it; an `is_ssh` early-return in
// `try_handle_dropped_paths_paste` used to skip the classifier and leave the
// path as literal text.
//
// This has to be a PTY case rather than a unit test: `terminal_context()` is a
// process-wide static computed once from the environment, so a unit test cannot
// make the pager believe it is on an SSH host. Setting `SSH_CONNECTION` for the
// spawned pager is the only faithful reproduction.
#[allow(unused_imports)]
use super::common::*;

/// `SSH_CONNECTION` is the guard's trigger; the value is shape-only.
const SSH_CONNECTION_VALUE: &str = "10.0.0.1 51000 10.0.0.2 22";

/// Sentinel the prompt shows for an attached image chip.
const IMAGE_CHIP_SENTINEL: &str = "[Image #";

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn bracketed_image_path_attaches_on_an_ssh_host() {
    let content = ContentController::start().await.expect("start content");
    content.set_response(format!("{MOCK_RESPONSE_SENTINEL} ssh image paste turn."));

    let binary = pager_binary().expect("resolve pager binary");

    // A real PNG on the host the pager runs on, with the shape a terminal host
    // gives its clipboard-image temp files (`orca-paste-<ts>-<uuid>.png`).
    let image_dir = tempfile::tempdir().expect("temp dir for the pasted image");
    let image_path = image_dir
        .path()
        .join("orca-paste-1760000000000-6f1a2b3c-4d5e-6f70-8192-a3b4c5d6e7f8.png");
    std::fs::write(&image_path, png_bytes(10, 10)).expect("write png fixture");
    let pasted = image_path.to_string_lossy().to_string();

    // Model the remote pane: the pager sees an SSH connection.
    let mut harness = PtyHarness::spawn_with_content_env_ops_in_dir(
        &binary,
        DEFAULT_ROWS,
        DEFAULT_COLS,
        &content,
        &[],
        &[EnvOp::set("SSH_CONNECTION", SSH_CONNECTION_VALUE)],
        Some(content.home()),
    )
    .expect("spawn pager with SSH_CONNECTION");

    harness
        .wait_for_text(WELCOME_SCREEN_SENTINEL, WELCOME_TIMEOUT)
        .expect("welcome text");

    // The terminal-host paste shape: a bracketed-paste frame carrying the path.
    harness
        .inject_keys(format!("\x1b[200~{pasted}\x1b[201~").as_bytes())
        .expect("bracketed-paste the image path");

    // Before the fix the paste landed as literal path text and no chip appeared.
    harness
        .wait_for_text(IMAGE_CHIP_SENTINEL, Duration::from_secs(10))
        .expect("image path attaches as a chip on an SSH host");

    assert!(
        !harness.contains_text("panicked"),
        "pager panicked\nscreen:\n{}",
        harness.screen_contents()
    );

    harness.quit().expect("clean quit");
}

/// A genuine, decodable PNG. The paste classifier requires real image bytes —
/// it sniffs the MIME type and decodes the dimensions (`read_image_at_path` →
/// `decode_image_dimensions`) — so a hand-rolled header is not enough. Written
/// with zlib "stored" blocks so the test needs no encoder dependency.
fn png_bytes(width: u32, height: u32) -> Vec<u8> {
    let mut raw = Vec::with_capacity((height * (1 + width * 4)) as usize);
    for _ in 0..height {
        raw.push(0); // filter: none
        for _ in 0..width {
            raw.extend_from_slice(&[0x80, 0x40, 0x20, 0xff]);
        }
    }

    let mut png = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // 8-bit RGBA, deflate, no interlace
    png_chunk(&mut png, b"IHDR", &ihdr);
    png_chunk(&mut png, b"IDAT", &zlib_stored(&raw));
    png_chunk(&mut png, b"IEND", &[]);
    png
}

/// `length | type | data | crc32(type + data)`.
fn png_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let mut crc_input = kind.to_vec();
    crc_input.extend_from_slice(data);
    out.extend_from_slice(&crc32(&crc_input).to_be_bytes());
}

/// A zlib stream whose deflate payload uses only stored (uncompressed) blocks.
fn zlib_stored(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01]; // CMF/FLG: deflate, 32K window, no dict
    let mut offset = 0;
    while offset < data.len() {
        let take = (data.len() - offset).min(0xffff);
        let is_last = offset + take == data.len();
        out.push(if is_last { 1 } else { 0 });
        out.extend_from_slice(&(take as u16).to_le_bytes());
        out.extend_from_slice(&(!(take as u16)).to_le_bytes());
        out.extend_from_slice(&data[offset..offset + take]);
        offset += take;
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in data {
        a = (a + u32::from(byte)) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}
