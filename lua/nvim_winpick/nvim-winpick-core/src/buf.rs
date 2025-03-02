use anyhow::Context;
use nvim_oxi::api::Buffer;

pub(crate) fn load_file_to_hidden_buffer(path: &str) -> anyhow::Result<Buffer> {
    let buf: Buffer = nvim_oxi::api::call_function("bufadd", (path,))
        .with_context(|| format!("failed to open buffer at {path}"))?;
    Ok(buf)
}
