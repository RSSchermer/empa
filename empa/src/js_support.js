export function __empa_js_copy_buffer_to_memory(
    byte_buffer,
    offset,
    size,
    wasm_memory,
    pointer,
) {
    // Create a view the relevant region of source buffer
    let range_view = new Uint8Array(byte_buffer.buffer, offset, size);

    // View the WASM memory buffer as bytes
    let memory_bytes = new Uint8Array(wasm_memory.buffer);

    // Copy the source range to WASM memory at the pointer location
    memory_bytes.set(range_view, pointer);
}
