mod file_server;

use file_server::{FileServer, Tree};

fn run() -> Result<(), String> {
    let data_dir_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data");

    let server = FileServer::from_dir(data_dir_path, &["txt"], 1024)?;

    let files = server.list_files().collect::<Vec<_>>();

    println!("Files available on the server:\n");

    files.iter().for_each(|file| println!("{:#?}", file));

    let file_index = 1;
    let chunk_index = 5;

    let file_root_hash = *files
        .get(file_index)
        .ok_or_else(|| format!("file with index {} not found on server", file_index))?
        .root_hash;

    let (chunk_proof, chunk_data) = server
        .get_file_chunk(file_root_hash, chunk_index)
        .ok_or_else(|| format!("chunk with index {} not found in file", chunk_index))?;

    let is_chunk_valid = Tree::verify_proof(&chunk_data, file_root_hash, &chunk_proof);

    println!(
        "\nChunk #{} of file #{} is {}",
        chunk_index,
        file_index,
        if is_chunk_valid { "VALID" } else { "INVALID" }
    );

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {}", error)
    }
}
