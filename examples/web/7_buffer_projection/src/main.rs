use std::error::Error;

use arwa::console;
use arwa::window::window;
use empa::arwa::{NavigatorExt, RequestAdapterOptions};
use empa::buffer;
use empa::buffer::projection;
use empa::device::DeviceDescriptor;
use futures::FutureExt;

#[derive(Clone, Copy, PartialEq, Debug)]
struct Foo {
    a: u32,
    b: Bar,
}

// TODO: mapping offsets need to be aligned to 8 bytes, add automatic margins for offsets that are
// not aligned to 8 bytes?
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C, align(8))]
struct Bar {
    c: f32,
}

fn main() {
    arwa::spawn_local(render().map(|res| res.unwrap()));
}

async fn render() -> Result<(), Box<dyn Error>> {
    let window = window();
    let empa = window.navigator().empa();

    let adapter = empa
        .request_adapter(&RequestAdapterOptions::default())
        .await
        .ok_or("adapter not found")?;
    let device = adapter.request_device(&DeviceDescriptor::default()).await?;

    let buffer = device.create_buffer(
        Foo {
            a: 1,
            b: Bar { c: 2.0 },
        },
        buffer::Usages::copy_dst().and_map_read(),
    );

    buffer.map_read().await?;

    let a_projection = buffer.project_to::<u32>(projection!(Foo => a));
    let b_projection = buffer.project_to::<Bar>(projection!(Foo => b));
    let c_projection = b_projection.project_to::<f32>(projection!(Bar => c));

    console::log!("a", &*a_projection.mapped());
    console::log!("c", &*c_projection.mapped());

    buffer.unmap();

    Ok(())
}
